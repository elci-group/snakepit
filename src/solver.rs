use crate::pep440::Version;
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::rc::Rc;
use anyhow::Result;
use std::sync::{Arc, Mutex};

// Represents a package name
pub type PackageName = String;

// Represents a version constraint (simplified for now, will need full range support)
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Constraint {
    Any,
    Exact(Version),
    Range(Version, Version), // Min (inclusive), Max (exclusive)
    // TODO: Add more complex ranges (union, intersection)
}

impl Constraint {
    pub fn allows(&self, version: &Version) -> bool {
        match self {
            Constraint::Any => true,
            Constraint::Exact(v) => v == version,
            Constraint::Range(min, max) => version >= min && version < max,
        }
    }

    pub fn intersect(&self, other: &Constraint) -> Constraint {
        // Simplified intersection logic
        match (self, other) {
            (Constraint::Any, c) | (c, Constraint::Any) => c.clone(),
            (Constraint::Exact(v1), Constraint::Exact(v2)) => {
                if v1 == v2 { Constraint::Exact(v1.clone()) } else { Constraint::Range(v1.clone(), v1.clone()) } // Empty
            },
            // ... more logic needed for ranges
            _ => Constraint::Any, // Placeholder
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
        cause: Rc<Incompatibility>, // The incompatibility that forced this derivation
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

use crate::resolver::DependencyResolver;

// The Solver driver
pub struct Solver {
    root: PackageName,
    root_version: Version,
    incompatibilities: Vec<Rc<Incompatibility>>,
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
                let version = self.fetch_best_version(&package).await?;
                
                // Add dependencies as incompatibilities
                let deps = self.fetch_dependencies(&package, &version).await?;
                for (dep_name, dep_constraint) in deps {
                    let term1 = Term::new(package.clone(), Constraint::Exact(version.clone()));
                    let term2 = Term::new(dep_name.clone(), dep_constraint).negate();
                    
                    self.incompatibilities.push(Rc::new(Incompatibility {
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

    fn resolve_conflict(&mut self, mut conflict: Rc<Incompatibility>) -> Result<()> {
        if conflict.terms.iter().any(|t| t.package == self.root) && self.solution.decision_level() == 0 {
            return Err(anyhow::anyhow!("Unsolvable conflict: {:?}", conflict));
        }

        let current_level = self.solution.decision_level();
        if current_level == 0 {
             return Err(anyhow::anyhow!("Unsolvable conflict at root: {:?}", conflict));
        }
        
        let backtrack_level = current_level - 1;
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

    async fn fetch_best_version(&self, package: &str) -> Result<Version> {
        let mut resolver = self.resolver.lock().unwrap();
        let info = resolver.fetch_package_info(package).await?;
        
        // Find latest version (simplified)
        // In real PubGrub, we'd pick the best version matching current constraints
        // For now, just pick the latest one that parses
        let mut best_version = None;
        for v_str in info.releases.keys() {
            if let Ok(v) = Version::parse(v_str) {
                if best_version.as_ref().map_or(true, |best| &v > best) {
                    best_version = Some(v);
                }
            }
        }
        
        best_version.ok_or_else(|| anyhow::anyhow!("No valid versions found for {}", package))
    }

    async fn fetch_dependencies(&self, package: &str, version: &Version) -> Result<Vec<(PackageName, Constraint)>> {
        let mut resolver = self.resolver.lock().unwrap();
        let info = resolver.fetch_package_info(package).await?;
        
        let mut deps = Vec::new();
        if let Some(requires) = &info.info.requires_dist {
            for req_str in requires {
                // Use PEP 508 parser
                if let Ok(spec) = crate::markers::parse_requirement(req_str) {
                    // Skip if marker doesn't match (simple check)
                    if let Some(marker) = &spec.marker {
                        let target_env = crate::markers::TargetEnvironment::default();
                        if !marker.evaluate(&target_env) {
                            continue; // Skip this dependency
                        }
                    }
                    
                    // Convert version specs to Constraint
                    let constraint = if spec.version_specs.is_empty() {
                        Constraint::Any
                    } else {
                        // For now, take the first constraint
                        // Full implementation would need to handle multiple constraints
                        let vspec = &spec.version_specs[0];
                        match vspec.operator.as_str() {
                            "==" => {
                                if let Ok(v) = Version::parse(&vspec.version) {
                                    Constraint::Exact(v)
                                } else {
                                    Constraint::Any
                                }
                            },
                            ">=" => {
                                if let Ok(min) = Version::parse(&vspec.version) {
                                    // Create a Range with a very high max
                                    let max = Version {
                                        epoch: 9999,
                                        release: vec![9999, 9999, 9999],
                                        pre: None,
                                        post: None,
                                        dev: None,
                                        local: None,
                                    };
                                    Constraint::Range(min, max)
                                } else {
                                    Constraint::Any
                                }
                            },
                            _ => Constraint::Any, // TODO: Handle other operators
                        }
                    };
                    
                    deps.push((spec.name, constraint));
                }
            }
        }
        
        Ok(deps)
    }

    fn propagate(&mut self) -> Option<Rc<Incompatibility>> {
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
