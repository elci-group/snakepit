use crate::pep440::Version;
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use tokio::sync::Mutex;

// Represents a package name
pub type PackageName = String;

pub type IncompatibilityRef = Arc<Incompatibility>;

// Represents a version constraint (simplified for now, will need full range support)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Constraint {
    Any,
    Exact(Version),
    Range(Version, Version), // Min (inclusive), Max (exclusive)
    Union(Vec<Constraint>),
    Intersection(Vec<Constraint>),
    Not(Box<Constraint>),
}

impl Constraint {
    pub fn allows(&self, version: &Version) -> bool {
        match self {
            Constraint::Any => true,
            Constraint::Exact(v) => v == version,
            Constraint::Range(min, max) => version >= min && version < max,
            Constraint::Union(constraints) => constraints.iter().any(|c| c.allows(version)),
            Constraint::Intersection(constraints) => constraints.iter().all(|c| c.allows(version)),
            Constraint::Not(c) => !c.allows(version),
        }
    }

    pub fn is_empty(&self) -> bool {
        match self {
            Constraint::Any => false,
            Constraint::Exact(_) => false,
            Constraint::Range(min, max) => min >= max,
            Constraint::Union(constraints) => constraints.iter().all(|c| c.is_empty()),
            Constraint::Intersection(constraints) => constraints.iter().any(|c| c.is_empty()), // Simplified
            Constraint::Not(c) => matches!(c.as_ref(), Constraint::Any), // Not(Any) is empty
        }
    }

    pub fn intersect(&self, other: &Constraint) -> Constraint {
        match (self, other) {
            (Constraint::Any, c) | (c, Constraint::Any) => c.clone(),
            (Constraint::Exact(v1), Constraint::Exact(v2)) => {
                if v1 == v2 { Constraint::Exact(v1.clone()) } else { Constraint::Union(vec![]) } // Empty
            },
            (Constraint::Range(min1, max1), Constraint::Range(min2, max2)) => {
                let new_min = if min1 > min2 { min1 } else { min2 };
                let new_max = if max1 < max2 { max1 } else { max2 };
                if new_min < new_max {
                    Constraint::Range(new_min.clone(), new_max.clone())
                } else {
                    Constraint::Union(vec![]) // Empty
                }
            },
            (Constraint::Intersection(c1), Constraint::Intersection(c2)) => {
                let mut combined = c1.clone();
                combined.extend(c2.clone());
                Constraint::Intersection(combined)
            },
            (c1, Constraint::Intersection(c2)) => {
                let mut combined = c2.clone();
                combined.push(c1.clone());
                Constraint::Intersection(combined)
            },
            (Constraint::Intersection(c1), c2) => {
                let mut combined = c1.clone();
                combined.push(c2.clone());
                Constraint::Intersection(combined)
            },
            (c1, c2) => Constraint::Intersection(vec![c1.clone(), c2.clone()]),
        }
    }
}

// A Term represents a statement about a package version: "Package A is selected and satisfies Constraint C"
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Term {
    pub package: PackageName,
    pub constraint: Constraint,
    pub positive: bool, // true = "is selected", false = "is NOT selected" (or satisfies constraint)
}

impl Term {
    pub fn new(package: PackageName, constraint: Constraint) -> Self {
        Self { package, constraint, positive: true }
    }
    
    pub fn negate(&self) -> Self {
        Self {
            package: self.package.clone(),
            constraint: self.constraint.clone(),
            positive: !self.positive,
        }
    }
    
    // Check relation with another term (subset, disjoint, overlap)
    pub fn relation(&self, other: &Term) -> SetRelation {
        if self.package != other.package {
            return SetRelation::Disjoint; 
        }
        
        // Simplified relation logic
        SetRelation::Overlapping
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SetRelation {
    Subset,
    Disjoint,
    Overlapping,
}

// An Incompatibility represents a set of terms that cannot all be true.
// e.g. { A, B } means "A and B cannot both be true".
// Usually derived from dependencies: "A depends on B" -> { A, not B }
#[derive(Debug, Clone)]
pub struct Incompatibility {
    pub terms: Vec<Term>,
    pub cause: IncompatibilityCause,
}

#[derive(Debug, Clone)]
pub enum IncompatibilityCause {
    Dependency(PackageName, PackageName), // A depends on B
    Root, // Root package requirement
    NoVersion, // No version matches constraint
    Conflict, // Derived conflict
}

// An Assignment is a decision made by the solver
#[derive(Debug, Clone)]
pub enum Assignment {
    Decision {
        package: PackageName,
        version: Version,
        decision_level: usize,
    },
    Derivation {
        term: Term,
        cause: IncompatibilityRef, // The incompatibility that forced this derivation
        decision_level: usize,
    },
}

// The PartialSolution tracks the current state of assignments
pub struct PartialSolution {
    pub assignments: Vec<Assignment>,
    pub decisions: HashMap<PackageName, Version>, // Fast lookup for decided versions
}

impl PartialSolution {
    pub fn new() -> Self {
        Self {
            assignments: Vec::new(),
            decisions: HashMap::new(),
        }
    }

    pub fn assign(&mut self, assignment: Assignment) {
        match &assignment {
            Assignment::Decision { package, version, .. } => {
                self.decisions.insert(package.clone(), version.clone());
            }
            Assignment::Derivation { .. } => {
                // Derivations might narrow constraints, not necessarily pick a version yet
            }
        }
        self.assignments.push(assignment);
    }
    
    pub fn decision_level(&self) -> usize {
        // Count decisions
        self.assignments.iter().filter(|a| matches!(a, Assignment::Decision { .. })).count()
    }
    pub fn satisfies(&self, term: &Term) -> bool {
        if let Some(version) = self.decisions.get(&term.package) {
            let allows = term.constraint.allows(version);
            if term.positive { allows } else { !allows }
        } else {
            false
        }
    }

    pub fn unsatisfies(&self, term: &Term) -> bool {
        if let Some(version) = self.decisions.get(&term.package) {
            let allows = term.constraint.allows(version);
            if term.positive { !allows } else { allows }
        } else {
            false
        }
    }
    
    pub fn backtrack(&mut self, decision_level: usize) {
        while self.decision_level() > decision_level {
            if let Some(assignment) = self.assignments.pop() {
                match assignment {
                    Assignment::Decision { package, .. } => {
                        self.decisions.remove(&package);
                    }
                    _ => {}
                }
            }
        }
    }
}

use crate::resolver::{DependencyResolver, PyPIPackageInfo};

// The Solver driver
pub struct Solver {
    root: PackageName,
    root_version: Version,
    incompatibilities: Vec<IncompatibilityRef>,
    solution: PartialSolution,
    resolver: Arc<Mutex<DependencyResolver>>,
}

impl Solver {
    pub fn new(root: PackageName, root_version: Version, resolver: Arc<Mutex<DependencyResolver>>) -> Self {
        Self {
            root,
            root_version,
            incompatibilities: Vec::new(),
            solution: PartialSolution::new(),
            resolver,
        }
    }

    pub async fn solve(&mut self) -> Result<HashMap<PackageName, Version>> {
        self.solution.assign(Assignment::Decision {
            package: self.root.clone(),
            version: self.root_version.clone(),
            decision_level: 0,
        });

        loop {
            if let Some(conflict) = self.propagate() {
                if let Err(e) = self.resolve_conflict(conflict) {
                    return Err(e); // Unsolvable
                }
                continue; // Retry propagation after backtracking
            }
            
            if let Some(package) = self.choose_next_package() {
                let info = self.fetch_package_info_safe(&package).await?;
                let version = self.find_best_version_from_info(&package, &info)?;
                let deps = self.extract_dependencies_from_info(&info)?;
                
                for (dep_name, dep_constraint) in deps {
                    let term1 = Term::new(package.clone(), Constraint::Exact(version.clone()));
                    let term2 = Term::new(dep_name.clone(), dep_constraint).negate();
                    
                    self.incompatibilities.push(Arc::new(Incompatibility {
                        terms: vec![term1, term2],
                        cause: IncompatibilityCause::Dependency(package.clone(), dep_name),
                    }));
                }

                self.solution.assign(Assignment::Decision {
                    package,
                    version,
                    decision_level: self.solution.decision_level() + 1,
                });
            } else {
                break; // Done
            }
        }

        Ok(self.solution.decisions.clone())
    }

    async fn fetch_package_info_safe(&self, package: &str) -> Result<PyPIPackageInfo> {
        let resolver = self.resolver.lock().await;
        let info = resolver.fetch_package_info(package).await?;
        Ok(info)
    }

    fn find_best_version_from_info(&self, package: &str, info: &PyPIPackageInfo) -> Result<Version> {
        // Collect constraints from solution
        let mut required_constraint = Constraint::Any;
        for assignment in &self.solution.assignments {
             match assignment {
                Assignment::Derivation { term, .. } => {
                    if term.package == package && term.positive {
                        required_constraint = required_constraint.intersect(&term.constraint);
                    }
                }
                _ => {}
             }
        }
        
        // Find latest version that satisfies the constraint
        let mut best_version = None;
        for v_str in info.releases.keys() {
            if let Ok(v) = Version::parse(v_str) {
                if required_constraint.allows(&v) {
                    if best_version.as_ref().map_or(true, |best| &v > best) {
                        best_version = Some(v);
                    }
                }
            }
        }
        
        best_version.ok_or_else(|| anyhow::anyhow!("No valid versions found for {} satisfying {:?}", package, required_constraint))
    }

    fn extract_dependencies_from_info(&self, info: &PyPIPackageInfo) -> Result<Vec<(PackageName, Constraint)>> {
        let mut deps = Vec::new();
        if let Some(requires) = &info.info.requires_dist {
            for req_str in requires {
                if let Ok(spec) = crate::markers::parse_requirement(req_str) {
                    if let Some(marker) = &spec.marker {
                        let target_env = crate::markers::TargetEnvironment::default();
                        if !marker.evaluate(&target_env) {
                            continue;
                        }
                    }
                    
                    let mut constraints = Vec::new();
                    if spec.version_specs.is_empty() {
                        constraints.push(Constraint::Any);
                    } else {
                        for vspec in &spec.version_specs {
                            let c = match vspec.operator.as_str() {
                                "==" => Version::parse(&vspec.version).map(Constraint::Exact).unwrap_or(Constraint::Any),
                                ">=" => Version::parse(&vspec.version).map(|min| {
                                    Constraint::Range(min, Version { epoch: 9999, release: vec![9999], pre: None, post: None, dev: None, local: None })
                                }).unwrap_or(Constraint::Any),
                                "<=" => Version::parse(&vspec.version).map(|v| {
                                    Constraint::Union(vec![
                                        Constraint::Range(Version { epoch: 0, release: vec![0], pre: None, post: None, dev: None, local: None }, v.clone()),
                                        Constraint::Exact(v)
                                    ])
                                }).unwrap_or(Constraint::Any),
                                ">" => Version::parse(&vspec.version).map(|v| {
                                    Constraint::Intersection(vec![
                                        Constraint::Range(v.clone(), Version { epoch: 9999, release: vec![9999], pre: None, post: None, dev: None, local: None }),
                                        Constraint::Not(Box::new(Constraint::Exact(v)))
                                    ])
                                }).unwrap_or(Constraint::Any),
                                "<" => Version::parse(&vspec.version).map(|v| {
                                    Constraint::Range(Version { epoch: 0, release: vec![0], pre: None, post: None, dev: None, local: None }, v)
                                }).unwrap_or(Constraint::Any),
                                "!=" => Version::parse(&vspec.version).map(|v| Constraint::Not(Box::new(Constraint::Exact(v)))).unwrap_or(Constraint::Any),
                                "~=" => {
                                    if let Ok(v) = Version::parse(&vspec.version) {
                                        if v.release.len() < 2 {
                                            Constraint::Any
                                        } else {
                                            let mut prefix = v.release.clone();
                                            prefix.pop();
                                            if let Some(last) = prefix.last_mut() { *last += 1; }
                                            let upper = Version { epoch: v.epoch, release: prefix, pre: None, post: None, dev: None, local: None };
                                            Constraint::Intersection(vec![
                                                Constraint::Range(v, Version { epoch: 9999, release: vec![9999], pre: None, post: None, dev: None, local: None }),
                                                Constraint::Range(Version { epoch: 0, release: vec![0], pre: None, post: None, dev: None, local: None }, upper)
                                            ])
                                        }
                                    } else { Constraint::Any }
                                },
                                _ => Constraint::Any,
                            };
                            constraints.push(c);
                        }
                    }
                    let final_c = if constraints.is_empty() { Constraint::Any }
                                 else if constraints.len() == 1 { constraints[0].clone() }
                                 else { Constraint::Intersection(constraints) };
                    deps.push((spec.name, final_c));
                }
            }
        }
        Ok(deps)
    }

    fn resolve_conflict(&mut self, conflict: IncompatibilityRef) -> Result<()> {
        let mut incompatibility = conflict;
        let mut created_incompatibility = false;

        loop {
            // Find the term that was satisfied last
            let mut most_recent_term = None;
            let mut most_recent_index = 0;
            let mut most_recent_satisfier = None;


            for term in &incompatibility.terms {
                // Find the assignment that satisfied this term
                // We search backwards
                for (i, assignment) in self.solution.assignments.iter().enumerate().rev() {
                    let satisfied = match assignment {
                        Assignment::Decision { package, version, .. } => {
                            package == &term.package && term.constraint.allows(version)
                        }
                        Assignment::Derivation { term: derived_term, .. } => {
                            derived_term.package == term.package && 
                            derived_term.positive == term.positive && // Simplified check
                            // Real check: derived_term implies term
                            // For now assume exact match or simple subset
                            true 
                        }
                    };

                    if satisfied {
                        if i >= most_recent_index {
                            most_recent_index = i;
                            most_recent_term = Some(term.clone());
                            most_recent_satisfier = Some(assignment.clone());
                        }
                        break;
                    }
                }
            }

            // If we can't find a satisfier, something is wrong (root conflict?)
            if most_recent_term.is_none() {
                 return Err(anyhow::anyhow!("Root conflict detected (no satisfier found): {:?}", incompatibility));
            }

            let satisfier = most_recent_satisfier.unwrap();
            
            // If the satisfier is a Decision, we found the root cause at this level
            // But we need to check if we are at the right level to backtrack
            let decision_level = match &satisfier {
                Assignment::Decision { decision_level, .. } => *decision_level,
                Assignment::Derivation { decision_level, .. } => *decision_level,
            };

            if decision_level == 0 {
                return Err(anyhow::anyhow!("Unsolvable conflict at root: {:?}", incompatibility));
            }

            match satisfier {
                Assignment::Decision { .. } => {
                    // We found the decision that caused the conflict.
                    // We need to backtrack.
                    break; 
                }
                Assignment::Derivation { cause, .. } => {
                    // Merge the incompatibility with the cause
                    // New terms = (incompatibility terms - term) U (cause terms - satisfier)
                    // This is the "Resolution" step
                    
                    let term_to_remove = most_recent_term.unwrap();
                    let mut new_terms = Vec::new();
                    
                    for t in &incompatibility.terms {
                        if t.package != term_to_remove.package {
                            new_terms.push(t.clone());
                        }
                    }
                    
                    for t in &cause.terms {
                        if t.package != term_to_remove.package {
                            // Check if we already have a term for this package
                            if let Some(existing_idx) = new_terms.iter().position(|x| x.package == t.package) {
                                // Intersect constraints
                                let existing = &new_terms[existing_idx];
                                let new_constraint = existing.constraint.intersect(&t.constraint);
                                new_terms[existing_idx] = Term::new(t.package.clone(), new_constraint);
                            } else {
                                new_terms.push(t.clone());
                            }
                        }
                    }
                    
                    incompatibility = Arc::new(Incompatibility {
                        terms: new_terms,
                        cause: IncompatibilityCause::Conflict,
                    });
                    created_incompatibility = true;
                }
            }
        }

        // Determine backtrack level
        // It's the highest decision level among the terms in the *new* incompatibility, 
        // excluding the one that was just resolved (which is now gone or changed).
        // Actually, in PubGrub, we backtrack to the level where the incompatibility becomes unit.
        // This is effectively the second highest decision level.
        
        let mut levels = Vec::new();
        for term in &incompatibility.terms {
             for assignment in self.solution.assignments.iter().rev() {
                 let satisfied = match assignment {
                    Assignment::Decision { package, version, .. } => {
                        package == &term.package && term.constraint.allows(version)
                    }
                    Assignment::Derivation { term: derived_term, .. } => {
                        derived_term.package == term.package
                    }
                };
                if satisfied {
                    match assignment {
                        Assignment::Decision { decision_level, .. } => levels.push(*decision_level),
                        Assignment::Derivation { decision_level, .. } => levels.push(*decision_level),
                    }
                    break;
                }
             }
        }
        
        levels.sort_unstable();
        levels.dedup();
        
        let backtrack_level = if levels.len() < 2 {
            0
        } else {
            levels[levels.len() - 2]
        };

        if created_incompatibility {
            self.incompatibilities.push(incompatibility);
        }

        self.solution.backtrack(backtrack_level);
        
        Ok(())
    }

    fn choose_next_package(&self) -> Option<PackageName> {
        for assignment in &self.solution.assignments {
            match assignment {
                Assignment::Derivation { term, .. } => {
                    if term.positive && !self.solution.decisions.contains_key(&term.package) {
                        return Some(term.package.clone());
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn propagate(&mut self) -> Option<IncompatibilityRef> {
        let mut changed = true;
        while changed {
            changed = false;
            for incompatibility in &self.incompatibilities {
                let mut satisfied_count = 0;
                let mut undecided_term = None;
                let mut failed = false;

                for term in &incompatibility.terms {
                    if self.solution.satisfies(term) {
                        satisfied_count += 1;
                    } else if self.solution.unsatisfies(term) {
                        failed = true;
                        break;
                    } else {
                        if undecided_term.is_some() {
                            failed = true; // More than one undecided
                            break;
                        }
                        undecided_term = Some(term);
                    }
                }

                if failed {
                    continue;
                }

                if let Some(term) = undecided_term {
                    // One undecided, others satisfied -> Propagate negation
                    self.solution.assign(Assignment::Derivation {
                        term: term.negate(),
                        cause: incompatibility.clone(),
                        decision_level: self.solution.decision_level(),
                    });
                    changed = true;
                } else if satisfied_count == incompatibility.terms.len() {
                    // All satisfied -> Conflict
                    return Some(incompatibility.clone());
                }
            }
        }
        None
    }
}
