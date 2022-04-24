//! Bookkeeping for Postgres type informations
//!
//! # Definitions
//!
//! TODO: Redundant with registry definitions.
//!
//! ## Postgres type
//!
//! A description for the semantics, serialization and contraints of a group of
//! values manipulated by Postgres.
//!
//! A type has two main parts:
//! - An identity: a canonical name and an internal object identifier (OID)
//! - A kind: the meaning of the type itself (e.g. "primitive", "array of T", etc.).
//!
//! The kind may refer to other Postgres types, forming a dependency graph
//! between types.
//!
//! When synchronizing information between the local Rust program and the
//! remote Postgres database, SQLx uses different types to represent how much
//! is known at a given point in time.
//!
//! # Type reference
//!
//! A (potentially incomplete) type identity: type OID, or type name, or both.
//! If the reference is valid, it unambiguously identifies a type.
//!
//! # Fetched type
//!
//! A complete identity (OID and type name) and a kind. If the kind has type
//! dependencies, only their reference is known. There are no guarantees
//! about the validity of the references.
//!
//! This is what you get when querying the database.
//!
//! ## Lazy type
//!
//! Either a type reference or a fetched type. This hides whether a type is
//! fetched already or not.
//!
//! This is what SQLx uses as the `TypeInfo` for Postgres.
//!
//! # Resolved type
//!
//! A type is resolved when we have full knowledge of about it and ALL its
//! its dependencies.

use crate::ext::ustr::UStr;
use crate::postgres::type_info2::PgType;
use crate::postgres::type_info2::{PgBuiltinType, PgTypeOid};
use crate::HashMap;
use ahash::AHashSet;
use std::fmt;
use thiserror::Error;

/// Local state of the Postgres catalog.
///
/// This objects is the central point for synchronization between the Rust
/// program and remote database for objects in the `pg_catalog` namespace.
/// It acts as a cache with a high level API to retrieve information about the
/// data queried from the database.
///
/// It is used in particular to support namespaces and custom types.
/// (Actually it does not support namespaces yet, but it should get this feature
/// at some point before SQLx 1.0).
///
/// # Types
///
/// Postgres supports an advanced type system with primitives and composite
/// types such as arrays or records. Besides the builtin types, users may also
/// define their own custom types. SQLx needs to resolve information about
/// these type to properly process data (encode/decode).
///
/// # Definitions
///
/// The registry uses the following vocabulary to represents its level of
/// knowledge about a type:
/// - Declared: The Rust programm has a type reference (name or oid), but we
///   don't know anything more about the type. The reference may even be invalid.
/// - Missing: We queried the database and there was no type for the
///   corresponding reference.
/// - Fetched: We queried the database and know the oid, name and kind.
///   There is no guarantee about the dependencies.
/// - Resolved: The type and all its dependencies are fetched.
///
/// # Assumptions
///
/// A core issue here is synchronizing the type information between the local
/// type registry in the Rust program and the actual types in the Postgres
/// database.
///
/// SQLx assumes two things:
/// 1. **Builtin Types**. The database contains [standard types from the default
///    catalog](https://www.postgresql.org/docs/14/datatype.html#DATATYPE-TABLE),
///    with their default names and ids.
/// 2. **Immutable Types**. When querying the DB for type information, the
///    result can be cached forever; or until `.clear` is called explicitly.
///    The types are not created, deleted, or modified in any way at runtime.
///
/// ## Safety
///
/// Both of these assumptions are required so SQLx can properly communicate
/// with the database. They should hold for the very vast majority of programs.
/// Breaking an invariant is safe in the Rust sense, but it may trigger a panic
/// or apply unexpected changes to the database.
///
/// **If you perform any request modifying the types in the database, make sure
/// to clear the local type registry**.
///
/// # Model
///
/// The catalog works in 3 steps:
/// 1. References: The Rust program declares objects it wishes to use using
///    references. For types, this corresponds to `PgTypeRef`. A reference
///    acts as **the input to DB queries**. The local registry keeps track of
///    references and their state: merely declared, or already retrieved from
///    the database.
///
/// 2. Cache: Queries are handled outside of the catalog. Once the result is
///    known, it may be inserted.
///
/// 3. Analysis: The local catalog also maintains some higher-level analysis
///    about the stored objects. In particular, it analyses type dependencies
///    to check if types are fully resolved.
///
///
///
/// The registry uses the same representation as Postgres: types may depend on
/// each other; as such, they form a potentially cyclic dependency graph.
///
/// Types such as `INT4`, `TEXT`, `ANY` or enums are leaf nodes (or "primitives").
/// Arrays, domain types, composite types, and ranges are advanced types: they
/// depend on other types. SQLx has full support for complex relationships
/// between types, supporting deeply nested types or even self-referential
/// types.
///
/// Nodes in this graph are represented by `PgType`. Edges corresponds to
/// type dependencies, you may retrieve them with `PgType::type_dependencies`.
///
/// # Resolution
///
/// The type registry supports lazy resolution. You may declare that a type
/// exists by name or oid, and defer its full resolution (id/name/kind) to a
/// later point.
///
/// The registry keeps track of the resolution state of all its dependencies.
/// - `get` lets you retrieve shallow types, with potentially missing
///   dependencies.
/// - `resolve` returns types checked to be deeply resolved (any reachable type
///   dependency is present in the registry).
///
/// # Limitations
///
/// The current implementation of the registry has two main limitations:
/// 1. You can't serialize/deserialize the registry. This prevents you from
///    resolving the registry and storing it in a file. The reason for this is
///    that it heavily uses OID values to identify types but they are not stable
///    across database resets.
/// 2. There is no support for mutations. If you change already fetched types
///    in the database, you must fully clear the registry.
/// 3. No namespace support. Two types with the same local name but in different
///    namespaces are not supported currently.
///
/// Solving issues 1/2 would probably require a more advanced model for
/// caching and incremental updates. Solving issue 3 requires to figure out
/// how to best represent namespaces (and make sure it's compatible with builtin
/// types).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct LocalPgCatalog {
    /// Map from type references to their value in the cache.
    ///
    /// See `ObjectRefState` for details.
    ///
    /// Part of the "reference" step.
    type_refs: HashMap<PgTypeRef, ObjectRefState<PgTypeCacheKey>>,

    /// Fetched type information
    ///
    /// This may be seen as a local counterpart to the `pg_catalog.pg_type`
    /// table, using the `oid` column as the cache key.
    /// SQLx only keeps track of the oid, full name and kind. The kind is
    /// enum describing the type, including its dependencies.
    ///
    /// Part of the "cache" step.
    pg_type_cache: HashMap<PgTypeCacheKey, PgType<PgTypeOid>>,
    // TODO: Keep track of namespaces
    // /// Fetched namespace information.
    // ///
    // /// This may be seen as a local counterpart to the
    // /// `pg_catalog.pg_namespace` table, using the `oid` column as the cache
    // /// key.
    // ///
    // /// Part of the "cache" step.
    // pg_namespace_cache: HashMap<PgNamespaceOid, ???>,
    type_resolutions: HashMap<PgTypeCacheKey, ResolutionState>,

    /// Type oid -> Type info (None if declared but unresolved yet)
    oid_to_type: HashMap<PgTypeOid, PgTypeState>,
    /// Map from dependency OID to corresponding resolutions waiting on it to resume
    pending_resolutions: HashMap<PgTypeOid, Vec<(PgTypeOid, PendingTypeResolution)>>,
}

/// Key used for cached postgres types.
///
/// This is currently defined as a mere alias over `PgTypeOid`, but when used
/// this way it should be treated as an opaque pointer into the catalog cache.
/// Avoiding indirection by an artificial cache key helps with troubleshooting.
/// We may consider using a fully opaque type / newtype wrapper if this proves
/// confusing.
/// Using an opaque cache key would unlock caching/serialization.
pub type PgTypeCacheKey = PgTypeOid;

/// A response from the database about an object in `LocalPgCatalog`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ObjectRefState<CacheKey> {
    /// The local program thinks that this object exists, but it was never
    /// queried from the database. No assumption can be made about the object.
    Declared,
    /// The object is NOT in the database.
    ///
    /// The database was queried and the object was not found.
    Missing,
    /// The object was successfully fetched from the database. The result is
    /// available in the `LocalPgCatalog` using the corresponding cache key.
    Fetched(CacheKey),
}

/// Current oid information for a given name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum RegistryOid {
    /// Name declared to exist by the Rust code, but the oid value is not known yet
    ///
    /// (never queried from the DB)
    Declared,
    /// The DB was queried, but no oid was found for this name.
    NotInDatabase,
    /// The DB was queried: the value of the oid is now resolved.
    Resolved(PgTypeOid),
}

impl Default for RegistryOid {
    fn default() -> Self {
        Self::Declared
    }
}

/// Current type information for a given oid.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum PgTypeState {
    /// OID declared to exist by the Rust code, but the type is not known yet
    ///
    /// (never queried from the DB)
    Declared,
    /// The DB was queried, but no type was found for this oid.
    Missing,
    /// The DB was queried, and the type was resolved for the corresponding name
    Fetched {
        /// Shallow type definition for this OID. Dependencies are not
        /// guaranteed to be present.
        ty: PgType<PgTypeOid>,
        /// Details about the
        state: ResolutionState,
    },
}

impl Default for PgTypeState {
    fn default() -> Self {
        Self::Declared
    }
}

/// For a type in the registry, current resolution state of its dependencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ResolutionState {
    /// Some transitive dependencies are not resolved yet.
    ///
    /// The associated `oid` corresponds to the current dependency preventing
    /// type resolution from moving forward.
    Partial(PgTypeOid),
    /// The type is fully resolved, including all its transitive dependencies.
    Full(DependencyGraphDepth),
    /// This type will _never_ be resolved: one its transitive is missing from
    /// the database. The argument the OID of the missing type.
    DependencyNotInDatabase(PgTypeOid),
}

/// Depth of a dependency graph.
///
/// - If a type has no dependencies, the depth of its dependency graph is `Finite(0)`.
/// - Otherwise, the depth is one more than the depth of the direct dependencies.
/// - If dependencies form a cycle, the depth is marked as `Circular`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum DependencyGraphDepth {
    Finite(usize),
    Circular,
}

impl DependencyGraphDepth {
    fn add_one(self) -> Self {
        match self {
            Self::Finite(d) => Self::Finite(d + 1),
            Self::Circular => Self::Circular,
        }
    }
}

impl LocalPgCatalog {
    /// Create a new local type registry.
    ///
    /// The new registry contains all [builtin types](PgBuiltinType) (it is not
    /// empty).
    pub(crate) fn new() -> Self {
        Self {
            type_refs: HashMap::new(),
            pg_type_cache: HashMap::new(),
            oid_to_type: HashMap::new(),
            pending_resolutions: HashMap::new(),
            type_resolutions: HashMap::new(),
        }
    }

    pub(crate) fn declare_type(&mut self, type_ref: PgTypeRef) {
        if let PgTypeRef::Oid(oid) = &type_ref {
            self.oid_to_type
                .entry(*oid)
                .or_insert(PgTypeState::Declared);
        }

        self.type_refs
            .entry(type_ref)
            .or_insert(ObjectRefState::Declared);
    }

    // TODO: Check for conflicts, return `Result`
    pub(crate) fn insert_type(&mut self, ty: PgType<PgTypeOid>) {
        for ty_dep in ty.type_dependencies() {
            self.declare_type(PgTypeRef::Oid(*ty_dep));
        }
        let oid = ty.oid();
        let cache_key: PgTypeCacheKey = oid;

        let fetched = ObjectRefState::Fetched(cache_key);
        self.type_refs.insert(PgTypeRef::Oid(ty.oid()), fetched);
        self.type_refs.insert(PgTypeRef::Name(ty.name()), fetched);
        self.type_refs
            .insert(PgTypeRef::OidAndName(ty.oid(), ty.name()), fetched);

        if &*ty.name() == "node2" {
            dbg!(&self);
        }

        self.oid_to_type.insert(
            cache_key,
            PgTypeState::Fetched {
                ty: ty.clone(),
                state: ResolutionState::Partial(oid),
            },
        );
        self.pg_type_cache.insert(cache_key, ty);

        self.pending_resolutions
            .entry(oid)
            .or_default()
            .push((oid, PendingTypeResolution::new(oid)));
        self.advance_resolutions(oid);
    }

    pub(crate) fn advance_resolutions(&mut self, initial: PgTypeOid) {
        let mut active: Vec<PgTypeOid> = vec![initial];
        while let Some(dep) = active.pop() {
            debug_assert!(
                matches!(
                    self.oid_to_type.get(&dep),
                    Some(PgTypeState::Fetched { .. })
                ),
                "freshly resolved dependency with oid {} is in the registry",
                dep
            );
            let pending = match self.pending_resolutions.remove(&dep) {
                Some(pending) => pending,
                None => continue,
            };
            for (oid, mut resolution) in pending {
                let new_state: ResolutionState = match resolution.resume(&self) {
                    GeneratorState::Yielded(new_dep) => {
                        debug_assert_ne!(new_dep, dep, "type resolution has moved forward");
                        self.pending_resolutions
                            .entry(new_dep)
                            .or_default()
                            .push((oid, resolution));
                        ResolutionState::Partial(new_dep)
                    }
                    GeneratorState::Complete(res) => {
                        // This oid is now fully resolved, add it to the active list
                        active.push(oid);
                        match res {
                            Ok(depth) => ResolutionState::Full(depth),
                            Err(dep) => ResolutionState::DependencyNotInDatabase(dep),
                        }
                    }
                };
                let state: &mut ResolutionState = match self.oid_to_type.get_mut(&oid) {
                    Some(PgTypeState::Fetched { state, .. }) => state,
                    _ => unreachable!("(BUG) Type resolution progressed but type is missing from the registry [oid={}]", oid),
                };
                debug_assert_eq!(
                    *state,
                    ResolutionState::Partial(dep),
                    "expected type to be partially resolved, stuck on oid={}",
                    dep
                );
                *state = new_state;
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
pub(crate) enum GetPgTypeError {
    #[error("never declared in the local type registry")]
    Undeclared,
    #[error(
        "never resolved from the database (despite being declared in the local type registry)"
    )]
    Unfetched,
    #[error("missing from the database (despite being declared in the local type registry)")]
    Missing,
}

impl LocalPgCatalog {
    /// Internal method to retrieve a type with its resolution state.
    pub(crate) fn get_by_oid_with_resolution(
        &self,
        oid: PgTypeOid,
    ) -> Result<(&PgType<PgTypeOid>, ResolutionState), GetPgTypeError> {
        if let Some(builtin) = PgBuiltinType::try_from_oid(oid) {
            return Ok((
                builtin.into_static_pg_type_with_oid(),
                ResolutionState::Full(DependencyGraphDepth::Finite(0)),
            ));
        }

        match self.oid_to_type.get(&oid) {
            None => Err(GetPgTypeError::Undeclared),
            Some(PgTypeState::Declared) => Err(GetPgTypeError::Unfetched),
            Some(PgTypeState::Missing) => Err(GetPgTypeError::Missing),
            Some(PgTypeState::Fetched { ty, state }) => Ok((ty, *state)),
        }
    }

    /// Get a shallowly-resolved type from the local registry
    pub(crate) fn get_type(
        &self,
        ty_ref: &PgTypeRef,
    ) -> Result<&PgType<PgTypeOid>, GetPgTypeError> {
        if let Some(builtin) = PgBuiltinType::try_from_ref(ty_ref) {
            return Ok(builtin.into_static_pg_type_with_oid());
        }

        match self.type_refs.get(ty_ref) {
            None => Err(GetPgTypeError::Undeclared),
            Some(ObjectRefState::Declared) => Err(GetPgTypeError::Unfetched),
            Some(ObjectRefState::Missing) => Err(GetPgTypeError::Missing),
            Some(ObjectRefState::Fetched(cache_key)) => match self.pg_type_cache.get(cache_key) {
                None => unreachable!(
                    "(BUG) {:?} is fetched but the value is missing from the cache",
                    ty_ref
                ),
                Some(cached) => Ok(cached),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Error)]
#[error("no fully-resolved type in the registry for the provided reference")]
pub(crate) struct ResolvePgTypeError {
    /// Type preventing the resolution of the input reference.
    ///
    /// This may be the input type itself, or any of its transitive dependencies.
    typ: PgTypeRef,
    /// Reason why `dep` is preventing resolution
    error: GetPgTypeError,
}

impl LocalPgCatalog {
    /// Get a deeply-resolved type from the local registry.
    pub(crate) fn resolve(
        &self,
        oid: PgTypeOid,
    ) -> Result<PgType<PgLiveTypeRef<'_>>, ResolvePgTypeError> {
        match self.get_by_oid_with_resolution(oid) {
            Ok((ty, state)) => match state {
                ResolutionState::Full(_) => {
                    Ok(ty.clone().map_dependencies(|ty_dep| PgLiveTypeRef {
                        registry: self,
                        type_ref: PgTypeRef::Oid(ty_dep),
                    }))
                }
                ResolutionState::Partial(blocker) => {
                    return Err(ResolvePgTypeError {
                        typ: PgTypeRef::Oid(blocker),
                        error: GetPgTypeError::Unfetched,
                    })
                }
                ResolutionState::DependencyNotInDatabase(missing) => {
                    return Err(ResolvePgTypeError {
                        typ: PgTypeRef::Oid(missing),
                        error: GetPgTypeError::Missing,
                    })
                }
            },
            Err(error) => {
                return Err(ResolvePgTypeError {
                    typ: PgTypeRef::Oid(oid),
                    error,
                })
            }
        }
    }
}

/// A reified type resolution.
///
/// This structure implements a resumable depth-first search over a type
/// dependency graph. It completes once all the transitive dependencies are
/// resolved, or a dependency is marked as missing from the database.
///
/// The API to resume the resolution is inspired by the nightly
/// [`Generator` trait](https://doc.rust-lang.org/std/ops/trait.Generator.html).
///
/// You may call `resolution.resume(&registry)` to advance the search.
/// - The resolution is paused when an unresolved dependency is encountered.
///   The dependency `oid` is returned then.
/// - When complete, the resolution returns a result:
///   - `Ok` with the depth of the dependency graph on success
///   - `Err` if the search found a type missing from the database, the payload
///     is the oid of the missing type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum PendingTypeResolution {
    /// Going through the Depth First Search of the type dependency graph.
    Search {
        visited: AHashSet<PgTypeOid>,
        // Vec<(parent_oid, node_oid)>
        stack: Vec<(Option<PgTypeOid>, PgTypeOid)>,
        max_depth: DependencyGraphDepth,
    },
    /// Traversal is complete:
    /// - On success, store the depth of the dependency chain
    /// - On error, store of oid of the transitive dependency missing from the
    ///   database.
    Complete(Result<DependencyGraphDepth, PgTypeOid>),
}

impl PendingTypeResolution {
    /// Create a new resolution starting at the type with the provided oid.
    pub(crate) fn new(oid: PgTypeOid) -> Self {
        Self::Search {
            visited: AHashSet::new(),
            stack: vec![(None, oid)],
            max_depth: DependencyGraphDepth::Finite(0),
        }
    }

    /// Resume resolution
    ///
    /// Move forward through the type dependency graph (Depth First Search
    /// traversal).
    /// - Resolution is suspended if an unresolved dependency is encountered.
    /// - Resolution completes when all the type dependencies are checked to
    ///   be in the registry, or one of the dependencies is missing from the
    ///   database.
    pub(crate) fn resume(
        &mut self,
        registry: &LocalPgCatalog,
    ) -> GeneratorState<PgTypeOid, Result<DependencyGraphDepth, PgTypeOid>> {
        'generator: loop {
            match self {
                Self::Search {
                    visited,
                    stack,
                    max_depth,
                } => {
                    // The code below implements a traditional iterative DFS.
                    while let Some((parent, top)) = stack.pop() {
                        let is_first_visit = visited.insert(top);
                        if !is_first_visit {
                            // Guard against diamond shapes and
                            // duplicates in `PgType::type_dependencies`
                            continue;
                        }
                        match registry.get_by_oid_with_resolution(top) {
                            Err(GetPgTypeError::Undeclared) => {
                                unreachable!("(BUG) Expected type dependencies to be declared in the registry, but [oid={}] is missing", top);
                            }
                            Err(GetPgTypeError::Unfetched) => {
                                // Revert changes in this iteration and suspend the resolution
                                visited.remove(&top);
                                stack.push((parent, top));
                                return GeneratorState::Yielded(top);
                            }
                            Err(GetPgTypeError::Missing) => {
                                *self = Self::Complete(Err(top));
                                continue 'generator;
                            }
                            Ok((ty, state)) => {
                                match state {
                                    ResolutionState::Full(depth) => {
                                        *max_depth = std::cmp::max(*max_depth, depth)
                                        // Skip recursion: this subgraph is already resolved
                                    }
                                    ResolutionState::DependencyNotInDatabase(missing) => {
                                        *self = Self::Complete(Err(missing));
                                        continue 'generator;
                                    }
                                    ResolutionState::Partial(_) => {
                                        // This is the "recursion" step of the traversal
                                        for ty_dep in ty.type_dependencies().rev() {
                                            if visited.contains(ty_dep) {
                                                *max_depth = DependencyGraphDepth::Circular;
                                            } else {
                                                stack.push((Some(top), *ty_dep));
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    *self = Self::Complete(Ok(max_depth.add_one()));
                    // Implicit `continue `generator`
                }
                Self::Complete(ret) => return GeneratorState::Complete(ret.clone()),
            }
        }
    }
}

/// Local version of the nightly [`GeneratorState`](https://doc.rust-lang.org/std/ops/enum.GeneratorState.html).
// TODO: Use the standard `GeneratorState` once it is stable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeneratorState<Y, R> {
    Yielded(Y),
    Complete(R),
}

/// A reference uniquely identifying a Postgres type.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum PgTypeRef {
    /// Internal type OID.
    Oid(PgTypeOid),
    /// Type name.
    Name(UStr),
    /// Internal type OID and type name.
    ///
    /// When retrieving a type through this variant, both values must match.
    OidAndName(PgTypeOid, UStr),
}

/// A fully-resolved Postgres type reference attached to a local type registry.
///
/// Any type reachable from it through the registry is guaranteed to be fully
/// resolved. In other words, all metadata was already queried from the database
/// to use this type.
#[derive(Clone, PartialEq, Eq)]
pub(crate) struct PgLiveTypeRef<'reg> {
    registry: &'reg LocalPgCatalog,
    type_ref: PgTypeRef,
}

impl<'reg> PgLiveTypeRef<'reg> {
    fn resolve(&self) -> PgType<PgLiveTypeRef<'_>> {
        let t = match self.registry.get_type(&self.type_ref) {
            Ok(t) => t,
            Err(e) => unreachable!(
                "(bug) PgLiveRef should always point to a resolved type [type_ref = {:?}]: {}",
                &self.type_ref, e
            ),
        };
        t.clone().map_dependencies(|type_ref| PgLiveTypeRef {
            registry: self.registry,
            type_ref: PgTypeRef::Oid(type_ref),
        })
    }
}

impl<'reg> fmt::Debug for PgLiveTypeRef<'reg> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "PgLiveTypeRef {{registry, type_ref: {:?}}}",
            &self.type_ref
        )
    }
}

/// Postgres composite kind details, checked to only use fully resolved dependencies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ResolvedPgCompositeKind<'reg> {
    /// Registry containing the resolved dependencies
    registry: &'reg LocalPgCatalog,
    /// Field list
    fields: &'reg [(String, PgTypeOid)],
}

impl<'reg> ResolvedPgCompositeKind<'reg> {
    pub(crate) fn fields(
        &self,
    ) -> impl Iterator<Item = (&str, PgLiveTypeRef<'reg>)> + DoubleEndedIterator + ExactSizeIterator
    {
        let registry = &self.registry;
        self.fields.iter().map(move |(name, ty)| {
            let ty = PgLiveTypeRef {
                registry,
                type_ref: PgTypeRef::Oid(*ty),
            };
            (name.as_str(), ty)
        })
    }
}

#[cfg(test)]
mod test {
    use crate::postgres::catalog::{
        GetPgTypeError, LocalPgCatalog, PgLiveTypeRef, PgTypeRef, ResolvePgTypeError,
    };
    use crate::postgres::type_info2::{
        ConstFromPgBuiltinType, PgBuiltinType, PgType, PgTypeKind, PgTypeOid,
    };

    #[test]
    fn test_empty_registry_has_builtin_types() {
        let registry = LocalPgCatalog::new();
        {
            let actual = registry.get_type(&PgTypeRef::Oid(PgBuiltinType::Bool.oid()));
            assert_eq!(actual, Ok(&PgType::BOOL));
        }
        {
            let actual = registry.get_type(&PgTypeRef::Oid(PgBuiltinType::BoolArray.oid()));
            assert_eq!(actual, Ok(&PgType::BOOL_ARRAY));
        }
    }

    #[test]
    fn test_custom_simple_type() {
        let mut registry = LocalPgCatalog::new();
        let oid = PgTypeOid::from_u32(10000);
        let typ = PgType {
            oid,
            name: "custom".into(),
            kind: PgTypeKind::Simple,
        };
        {
            let actual = registry.get_type(&PgTypeRef::Oid(oid));
            assert_eq!(actual, Err(GetPgTypeError::Undeclared));
        }
        registry.declare_type(PgTypeRef::Oid(oid));
        {
            let actual = registry.get_type(&PgTypeRef::Oid(oid));
            assert_eq!(actual, Err(GetPgTypeError::Unfetched));
        }
        registry.insert_type(typ.clone());
        {
            let actual = registry.get_type(&PgTypeRef::Oid(oid));
            assert_eq!(actual, Ok(&typ));
        }
        {
            let actual = registry.resolve(oid);
            assert_eq!(
                actual,
                Ok(PgType {
                    oid,
                    name: "custom".into(),
                    kind: PgTypeKind::Simple,
                })
            );
        }
    }

    #[test]
    fn test_int4_domain_type() {
        let mut registry = LocalPgCatalog::new();
        let oid = PgTypeOid::from_u32(10000);
        let typ = PgType {
            oid,
            name: "myint".into(),
            kind: PgTypeKind::Domain(PgBuiltinType::Int4.oid()),
        };
        {
            let actual = registry.get_type(&PgTypeRef::Oid(oid));
            assert_eq!(actual, Err(GetPgTypeError::Undeclared));
        }
        registry.declare_type(PgTypeRef::Oid(oid));
        {
            let actual = registry.get_type(&PgTypeRef::Oid(oid));
            assert_eq!(actual, Err(GetPgTypeError::Unfetched));
        }
        registry.insert_type(typ.clone());
        {
            let actual = registry.get_type(&PgTypeRef::Oid(oid));
            assert_eq!(actual, Ok(&typ));
        }
        {
            let actual = registry.resolve(oid);
            assert_eq!(
                actual,
                Ok(PgType {
                    oid,
                    name: "myint".into(),
                    kind: PgTypeKind::Domain(PgLiveTypeRef {
                        registry: &registry,
                        type_ref: PgTypeRef::Oid(PgBuiltinType::Int4.oid())
                    }),
                })
            );
        }
    }

    #[test]
    fn test_linked_list_of_int4_by_uuid() {
        let mut registry = LocalPgCatalog::new();
        let oid = PgTypeOid::from_u32(10000);
        let typ = PgType {
            oid,
            name: "node".into(),
            kind: PgTypeKind::composite(vec![
                ("value".to_string(), PgBuiltinType::Int4.oid()),
                ("next".to_string(), PgBuiltinType::Uuid.oid()),
            ]),
        };
        {
            let actual = registry.get_type(&PgTypeRef::Oid(oid));
            assert_eq!(actual, Err(GetPgTypeError::Undeclared));
        }
        registry.declare_type(PgTypeRef::Oid(oid));
        {
            let actual = registry.get_type(&PgTypeRef::Oid(oid));
            assert_eq!(actual, Err(GetPgTypeError::Unfetched));
        }
        registry.insert_type(typ.clone());
        {
            let actual = registry.get_type(&PgTypeRef::Oid(oid));
            assert_eq!(actual, Ok(&typ));
        }
        {
            let actual = registry.resolve(oid);
            assert_eq!(
                actual,
                Ok(PgType {
                    oid,
                    name: "node".into(),
                    kind: PgTypeKind::composite(vec![
                        (
                            "value".to_string(),
                            PgLiveTypeRef {
                                registry: &registry,
                                type_ref: PgTypeRef::Oid(PgBuiltinType::Int4.oid())
                            }
                        ),
                        (
                            "next".to_string(),
                            PgLiveTypeRef {
                                registry: &registry,
                                type_ref: PgTypeRef::Oid(PgBuiltinType::Uuid.oid())
                            }
                        ),
                    ],)
                })
            );
        }
    }

    #[test]
    fn test_linked_list_of_domain_by_uuid() {
        let mut registry = LocalPgCatalog::new();
        let domain_oid = PgTypeOid::from_u32(10000);
        let domain_typ = PgType {
            oid: domain_oid,
            name: "myint".into(),
            kind: PgTypeKind::Domain(PgBuiltinType::Int4.oid()),
        };
        let node_oid = PgTypeOid::from_u32(10001);
        let node_typ = PgType {
            oid: node_oid,
            name: "node".into(),
            kind: PgTypeKind::composite(vec![
                ("value".to_string(), domain_oid),
                ("next".to_string(), PgBuiltinType::Uuid.oid()),
            ]),
        };
        {
            let actual = registry.get_type(&PgTypeRef::Oid(domain_oid));
            assert_eq!(actual, Err(GetPgTypeError::Undeclared));
            let actual = registry.get_type(&PgTypeRef::Oid(node_oid));
            assert_eq!(actual, Err(GetPgTypeError::Undeclared));
        }
        registry.declare_type(PgTypeRef::Oid(node_oid));
        {
            let actual = registry.get_type(&PgTypeRef::Oid(domain_oid));
            assert_eq!(actual, Err(GetPgTypeError::Undeclared));
            let actual = registry.get_type(&PgTypeRef::Oid(node_oid));
            assert_eq!(actual, Err(GetPgTypeError::Unfetched));
        }
        registry.insert_type(node_typ.clone());
        {
            let actual = registry.get_type(&PgTypeRef::Oid(domain_oid));
            assert_eq!(actual, Err(GetPgTypeError::Unfetched));
            let actual = registry.get_type(&PgTypeRef::Oid(node_oid));
            assert_eq!(actual, Ok(&node_typ));
        }
        {
            let actual = registry.resolve(node_oid);
            assert_eq!(
                actual,
                Err(ResolvePgTypeError {
                    typ: PgTypeRef::Oid(domain_oid),
                    error: GetPgTypeError::Unfetched
                })
            );
        }
        registry.insert_type(domain_typ.clone());
        {
            let actual = registry.resolve(node_oid);
            assert_eq!(
                actual,
                Ok(PgType {
                    oid: node_oid,
                    name: "node".into(),
                    kind: PgTypeKind::composite(vec![
                        (
                            "value".to_string(),
                            PgLiveTypeRef {
                                registry: &registry,
                                type_ref: PgTypeRef::Oid(domain_oid)
                            }
                        ),
                        (
                            "next".to_string(),
                            PgLiveTypeRef {
                                registry: &registry,
                                type_ref: PgTypeRef::Oid(PgBuiltinType::Uuid.oid())
                            }
                        ),
                    ],)
                })
            );
        }
    }

    #[test]
    fn test_linked_list_of_int4_by_self() {
        let mut registry = LocalPgCatalog::new();
        let oid = PgTypeOid::from_u32(10000);
        let typ = PgType {
            oid,
            name: "node".into(),
            kind: PgTypeKind::composite(vec![
                ("value".to_string(), PgBuiltinType::Int4.oid()),
                ("next".to_string(), oid),
            ]),
        };
        {
            let actual = registry.get_type(&PgTypeRef::Oid(oid));
            assert_eq!(actual, Err(GetPgTypeError::Undeclared));
        }
        registry.declare_type(PgTypeRef::Oid(oid));
        {
            let actual = registry.get_type(&PgTypeRef::Oid(oid));
            assert_eq!(actual, Err(GetPgTypeError::Unfetched));
        }
        registry.insert_type(typ.clone());
        {
            let actual = registry.get_type(&PgTypeRef::Oid(oid));
            assert_eq!(actual, Ok(&typ));
        }
        {
            let actual = registry.resolve(oid);
            assert_eq!(
                actual,
                Ok(PgType {
                    oid,
                    name: "node".into(),
                    kind: PgTypeKind::composite(vec![
                        (
                            "value".to_string(),
                            PgLiveTypeRef {
                                registry: &registry,
                                type_ref: PgTypeRef::Oid(PgBuiltinType::Int4.oid())
                            }
                        ),
                        (
                            "next".to_string(),
                            PgLiveTypeRef {
                                registry: &registry,
                                type_ref: PgTypeRef::Oid(oid)
                            }
                        ),
                    ],)
                })
            );
        }
    }
}
