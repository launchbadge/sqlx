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
use crate::postgres::type_info as type_info1;
use crate::postgres::type_info2::{OwningPgCompositeKind, PgBuiltinType, PgTypeOid};
use crate::postgres::type_info2::{PgType, PgTypeKind};
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
    /// Current type resolution information for cached types.
    ///
    /// Part of the "analysis" step.
    type_resolutions: HashMap<PgTypeCacheKey, ResolutionState>,

    /// List of (root_type, type_resolution_generator) for pending type resolutions,
    /// partitioned by type oid they are currently stuck on.
    ///
    /// Part of the "analysis" step.
    pending_resolutions: HashMap<PgTypeOid, Vec<(PgTypeOid, PendingTypeResolution)>>,
}

/// Key used for cached postgres types.
///
/// This an opaque key into the type cache inside `LocalPgCatalog`. This
/// identifies a fetched type from the database.
///
/// This key has no particular meaning. It is not related to the OID or name.
/// This is a deliberate choice to enable serialization of the local cache
/// without relying on the stability of the names and OIDs. The main issue when
/// using OIDs as cache keys is that custom type OIDs change change across DB
/// resets.
// The documentation clearly mentions that the key is opaque. But in practice
// we still use the OID for now as we don't support offline caches so the
// stability issue is not a concern.
// This is a private implementation detail and may change at any time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PgTypeCacheKey(PgTypeOid);

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
    /// Create a new local catalog, populated with [builtin types](PgBuiltinType).
    ///
    /// Use [`empty`] to create a local catalog without any content.
    pub(crate) fn new() -> Self {
        let mut catalog = Self::empty();
        for ty in PgBuiltinType::iter() {
            catalog
                .insert_type(ty.into_static_pg_type_with_oid().clone())
                .expect("builtin type insertion should always succeed");
        }
        catalog
    }

    /// Create a new empty local catalog.
    ///
    /// Use [`new`] to create a local catalog populated with builtin types.
    pub(crate) fn empty() -> Self {
        Self {
            type_refs: HashMap::new(),
            pg_type_cache: HashMap::new(),
            pending_resolutions: HashMap::new(),
            type_resolutions: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum FlagTypeAsMissingError {
    /// Conflict detected when marking the reference `ty_ref` as missing.
    ///
    /// `old` matches `ty_ref`.
    #[error("marking the reference {ty_ref:?} as missing conflicts with the already cached type {old:?}")]
    Conflict {
        /// Type reference we are trying to mark as missing
        ty_ref: PgTypeRef,
        /// Old cached value for `ty_ref`.
        old: PgType<PgTypeOid>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum InsertTypeError {
    /// Conflict detected when inserting the type `new` relative to the reference `ty_ref`.
    ///
    /// `old` and `new` are different, but both match `ty_ref`.
    #[error("inserting the type {new:?} into the local catalog conflicts with cached data, the reference {ty_ref:?} is already associated with the different type {old:?}")]
    Conflict {
        /// Type reference matching both types
        ty_ref: PgTypeRef,
        /// Old cached value for `ty_ref`.
        ///
        /// `None` means that it is explicitly missing from the database. It is
        /// a conflict to try to insert it afterwards without clearing the cache.
        old: Option<PgType<PgTypeOid>>,
        /// New type we are trying to insert into the cache.
        ///
        /// Invariant: `Some(self.new) != self.old`
        new: PgType<PgTypeOid>,
    },
}

impl LocalPgCatalog {
    /// Mark a type reference as expected to exist in the remote database.
    ///
    /// If the type reference was already queried from the database, this has
    /// no effect.
    pub(crate) fn declare_type(&mut self, ty_ref: PgTypeRef) {
        self.type_refs
            .entry(ty_ref)
            .or_insert(ObjectRefState::Declared);
    }

    /// Mark a type reference as known to be missing from the remote database.
    ///
    /// This function is used to report "not found" errors back to the catalog.
    /// It helps ensuring the consistency of the data and providing better
    /// error messages.
    ///
    /// This function checks for conflicts with previous operations. A conflict
    /// occurs when the cache already contains a type matching the reference.
    /// On conflict, no change is applied.
    ///
    /// It is safe to flag the same reference as missing multiple times.
    pub(crate) fn flag_type_as_missing(
        &mut self,
        ty_ref: PgTypeRef,
    ) -> Result<(), FlagTypeAsMissingError> {
        match self.type_refs.get(&ty_ref) {
            Some(ObjectRefState::Fetched(old_key)) => {
                // Conflict: Already cached and pointing to an existing type
                let old_ty = self
                    .pg_type_cache
                    .get(old_key)
                    .expect("(BUG) fetched type must exist in the cache");
                Err(FlagTypeAsMissingError::Conflict {
                    ty_ref,
                    old: old_ty.clone(),
                })
            }
            Some(ObjectRefState::Missing) => {
                // Already known as missing, nothing to do
                Ok(())
            }
            Some(ObjectRefState::Declared) | None => {
                // Declared or unknown, mark explicitly as missing
                let oid = ty_ref.as_oid();
                self.type_refs.insert(ty_ref, ObjectRefState::Missing);
                if let Some(oid) = oid {
                    self.advance_resolutions(oid);
                }
                Ok(())
            }
        }
    }

    /// Insert a new type in the local catalog
    ///
    /// This will cache the type and update type resolution information.
    ///
    /// This function checks for conflicts with previous operations. There are
    /// two situations for a conflict:
    /// 1. A reference was explicitly set as missing, but the new type matches it
    /// 2. The cache already contains a different type matching a reference
    ///    (e.g. same oid but different kind)
    /// On conflict, no change is applied.
    ///
    /// It is safe to insert exact duplicates.
    pub(crate) fn insert_type(&mut self, ty: PgType<PgTypeOid>) -> Result<(), InsertTypeError> {
        let refs = [
            PgTypeRef::Oid(ty.oid()),
            PgTypeRef::Name(ty.name()),
            PgTypeRef::OidAndName(ty.oid(), ty.name()),
        ];

        // Step 1: Check for conflicts
        let old_refs = refs
            .as_slice()
            .into_iter()
            .filter_map(|r| self.type_refs.get(r).map(|s| (r, s)));
        let mut old_ty_key: Option<PgTypeCacheKey> = None;
        let mut old_ty_key_matches: usize = 0;
        for (old_ref, state) in old_refs {
            match state {
                ObjectRefState::Declared => {
                    // The type was declared, it will be inserted now: no problem there
                }
                ObjectRefState::Missing => {
                    // Conflict: Previous query responded that the type is missing, but now it says it exists.
                    return Err(InsertTypeError::Conflict {
                        ty_ref: old_ref.clone(),
                        old: None,
                        new: ty,
                    });
                }
                ObjectRefState::Fetched(old_key) => {
                    let old_ty = self
                        .pg_type_cache
                        .get(old_key)
                        .expect("(BUG) fetched type must exist in the cache");
                    if *old_ty != ty {
                        // The ref (id or name or both) was already in the cache, but the new value
                        // and old value are different: the type changed!
                        return Err(InsertTypeError::Conflict {
                            ty_ref: old_ref.clone(),
                            old: Some(old_ty.clone()),
                            new: ty,
                        });
                    }
                    if let Some(prev_old_key) = old_ty_key {
                        if prev_old_key != *old_key {
                            unreachable!("(BUG) Inconsistent type cache when inserting `ty`: `cache[key1] == ty` and `cache[key2] == ty` but `key1` != `key2`. {:?}", (prev_old_key, old_key, ty));
                        }
                    }
                    old_ty_key = Some(*old_key);
                    old_ty_key_matches += 1;
                }
            }
        }

        // Step 2: Early exit to avoid duplicate insertion
        if let Some(old_ty_key) = old_ty_key {
            // If any duplicate is found, it MUST match over all references.
            if old_ty_key_matches != refs.len() {
                unreachable!("(BUG) Inconsistent type cache when inserting `ty`: found duplicate, but only part of the references were matched. {:?}", (old_ty_key, ty));
            }
            return Ok(());
        }

        // Step 3: Insert the type in the cache
        let oid = ty.oid();
        let cache_key = PgTypeCacheKey(oid);
        for ty_dep in ty.type_dependencies() {
            self.declare_type(PgTypeRef::Oid(*ty_dep));
        }
        for r in refs {
            self.type_refs.insert(r, ObjectRefState::Fetched(cache_key));
        }
        self.pg_type_cache.insert(cache_key, ty);
        self.type_resolutions
            .insert(cache_key, ResolutionState::Partial(oid));

        // Step 4: Advance analysis
        self.pending_resolutions
            .entry(oid)
            .or_default()
            .push((oid, PendingTypeResolution::new(oid)));
        self.advance_resolutions(oid);
        Ok(())
    }

    pub(crate) fn insert_legacy_type(
        &mut self,
        ty: &type_info1::PgTypeInfo,
    ) -> Result<(), InsertTypeError> {
        let mut stack: Vec<&type_info1::PgTypeInfo> = vec![ty];
        let mut visited: AHashSet<PgTypeOid> = AHashSet::new();
        while let Some(ty) = stack.pop() {
            let first_visit = visited.insert(ty.oid());
            if !first_visit {
                continue;
            }

            #[rustfmt::skip]
            let res = match &ty.0 {
                type_info1::PgType::Bool => self.insert_type(PgBuiltinType::Bool.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Bytea => self.insert_type(PgBuiltinType::Bytea.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Char => self.insert_type(PgBuiltinType::Char.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Name => self.insert_type(PgBuiltinType::Name.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Int8 => self.insert_type(PgBuiltinType::Int8.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Int2 => self.insert_type(PgBuiltinType::Int2.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Int4 => self.insert_type(PgBuiltinType::Int4.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Text => self.insert_type(PgBuiltinType::Text.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Oid => self.insert_type(PgBuiltinType::Oid.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Json => self.insert_type(PgBuiltinType::Json.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::JsonArray => self.insert_type(PgBuiltinType::JsonArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Point => self.insert_type(PgBuiltinType::Point.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Lseg => self.insert_type(PgBuiltinType::Lseg.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Path => self.insert_type(PgBuiltinType::Path.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Box => self.insert_type(PgBuiltinType::Box.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Polygon => self.insert_type(PgBuiltinType::Polygon.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Line => self.insert_type(PgBuiltinType::Line.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::LineArray => self.insert_type(PgBuiltinType::LineArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Cidr => self.insert_type(PgBuiltinType::Cidr.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::CidrArray => self.insert_type(PgBuiltinType::CidrArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Float4 => self.insert_type(PgBuiltinType::Float4.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Float8 => self.insert_type(PgBuiltinType::Float8.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Unknown => self.insert_type(PgBuiltinType::Unknown.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Circle => self.insert_type(PgBuiltinType::Circle.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::CircleArray => self.insert_type(PgBuiltinType::CircleArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Macaddr8 => self.insert_type(PgBuiltinType::Macaddr8.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Macaddr8Array => self.insert_type(PgBuiltinType::Macaddr8Array.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Macaddr => self.insert_type(PgBuiltinType::Macaddr.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Inet => self.insert_type(PgBuiltinType::Inet.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::BoolArray => self.insert_type(PgBuiltinType::BoolArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::ByteaArray => self.insert_type(PgBuiltinType::ByteaArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::CharArray => self.insert_type(PgBuiltinType::CharArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::NameArray => self.insert_type(PgBuiltinType::NameArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Int2Array => self.insert_type(PgBuiltinType::Int2Array.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Int4Array => self.insert_type(PgBuiltinType::Int4Array.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::TextArray => self.insert_type(PgBuiltinType::TextArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::BpcharArray => self.insert_type(PgBuiltinType::BpcharArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::VarcharArray => self.insert_type(PgBuiltinType::VarcharArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Int8Array => self.insert_type(PgBuiltinType::Int8Array.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::PointArray => self.insert_type(PgBuiltinType::PointArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::LsegArray => self.insert_type(PgBuiltinType::LsegArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::PathArray => self.insert_type(PgBuiltinType::PathArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::BoxArray => self.insert_type(PgBuiltinType::BoxArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Float4Array => self.insert_type(PgBuiltinType::Float4Array.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Float8Array => self.insert_type(PgBuiltinType::Float8Array.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::PolygonArray => self.insert_type(PgBuiltinType::PolygonArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::OidArray => self.insert_type(PgBuiltinType::OidArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::MacaddrArray => self.insert_type(PgBuiltinType::MacaddrArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::InetArray => self.insert_type(PgBuiltinType::InetArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Bpchar => self.insert_type(PgBuiltinType::Bpchar.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Varchar => self.insert_type(PgBuiltinType::Varchar.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Date => self.insert_type(PgBuiltinType::Date.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Time => self.insert_type(PgBuiltinType::Time.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Timestamp => self.insert_type(PgBuiltinType::Timestamp.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::TimestampArray => self.insert_type(PgBuiltinType::TimestampArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::DateArray => self.insert_type(PgBuiltinType::DateArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::TimeArray => self.insert_type(PgBuiltinType::TimeArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Timestamptz => self.insert_type(PgBuiltinType::Timestamptz.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::TimestamptzArray => self.insert_type(PgBuiltinType::TimestamptzArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Interval => self.insert_type(PgBuiltinType::Interval.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::IntervalArray => self.insert_type(PgBuiltinType::IntervalArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::NumericArray => self.insert_type(PgBuiltinType::NumericArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Timetz => self.insert_type(PgBuiltinType::Timetz.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::TimetzArray => self.insert_type(PgBuiltinType::TimetzArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Bit => self.insert_type(PgBuiltinType::Bit.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::BitArray => self.insert_type(PgBuiltinType::BitArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Varbit => self.insert_type(PgBuiltinType::Varbit.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::VarbitArray => self.insert_type(PgBuiltinType::VarbitArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Numeric => self.insert_type(PgBuiltinType::Numeric.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Record => self.insert_type(PgBuiltinType::Record.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::RecordArray => self.insert_type(PgBuiltinType::RecordArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Uuid => self.insert_type(PgBuiltinType::Uuid.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::UuidArray => self.insert_type(PgBuiltinType::UuidArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Jsonb => self.insert_type(PgBuiltinType::Jsonb.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::JsonbArray => self.insert_type(PgBuiltinType::JsonbArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Int4Range => self.insert_type(PgBuiltinType::Int4Range.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Int4RangeArray => self.insert_type(PgBuiltinType::Int4RangeArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::NumRange => self.insert_type(PgBuiltinType::NumRange.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::NumRangeArray => self.insert_type(PgBuiltinType::NumRangeArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::TsRange => self.insert_type(PgBuiltinType::TsRange.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::TsRangeArray => self.insert_type(PgBuiltinType::TsRangeArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::TstzRange => self.insert_type(PgBuiltinType::TstzRange.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::TstzRangeArray => self.insert_type(PgBuiltinType::TstzRangeArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::DateRange => self.insert_type(PgBuiltinType::DateRange.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::DateRangeArray => self.insert_type(PgBuiltinType::DateRangeArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Int8Range => self.insert_type(PgBuiltinType::Int8Range.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Int8RangeArray => self.insert_type(PgBuiltinType::Int8RangeArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Jsonpath => self.insert_type(PgBuiltinType::Jsonpath.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::JsonpathArray => self.insert_type(PgBuiltinType::JsonpathArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Money => self.insert_type(PgBuiltinType::Money.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::MoneyArray => self.insert_type(PgBuiltinType::MoneyArray.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Void => self.insert_type(PgBuiltinType::Void.into_static_pg_type_with_oid().clone()),
                type_info1::PgType::Custom(ty) => {
                    let oid = ty.oid;
                    let name = ty.name.clone();
                    let kind = match &ty.kind {
                        type_info1::PgTypeKind::Simple => PgTypeKind::Simple,
                        type_info1::PgTypeKind::Pseudo => PgTypeKind::Pseudo,
                        type_info1::PgTypeKind::Domain(ty) => {
                            stack.push(ty);
                            PgTypeKind::Domain(ty.oid())
                        },
                        type_info1::PgTypeKind::Composite(legacy_fields) => {
                            let mut fields: Vec<(String, PgTypeOid)> = Vec::with_capacity(legacy_fields.len());
                            for (key, ty) in legacy_fields.iter() {
                                stack.push(ty);
                                fields.push((key.clone(), ty.oid()));
                            }
                            PgTypeKind::Composite(OwningPgCompositeKind { fields: fields.into() })
                        }
                        type_info1::PgTypeKind::Array(ty) => {
                            stack.push(ty);
                            PgTypeKind::Array(ty.oid())
                        }
                        type_info1::PgTypeKind::Enum(variants) =>
                            PgTypeKind::Enum(variants.iter().cloned().collect::<Vec<_>>().into_boxed_slice()),

                        type_info1::PgTypeKind::Range(ty) => {
                            stack.push(ty);
                            PgTypeKind::Array(ty.oid())
                        }
                    };
                    self.insert_type(PgType {
                        oid,
                        name,
                        kind,
                    })
                }
                type_info1::PgType::DeclareWithName(name) => unreachable!("(bug) use of unresolved type declaration [name={}]", &name),
                type_info1::PgType::DeclareWithOid(oid) => unreachable!("(bug) use of unresolved type declaration [oid={}]", oid),
            };
            if let Err(e) = res {
                return Err(e);
            }
        }
        Ok(())
    }

    pub(crate) fn advance_resolutions(&mut self, initial: PgTypeOid) {
        let mut active: Vec<PgTypeOid> = vec![initial];
        while let Some(dep) = active.pop() {
            debug_assert!(
                matches!(
                    self.type_refs.get(&PgTypeRef::Oid(dep)),
                    Some(ObjectRefState::Fetched(_))
                ),
                "freshly resolved dependency with oid {} should be in the local catalog cache",
                dep
            );
            let pending = match self.pending_resolutions.remove(&dep) {
                Some(pending) => pending,
                None => continue,
            };
            for (oid, mut resolution) in pending {
                let new_state: ResolutionState = match resolution.resume(&self) {
                    GeneratorState::Yielded(new_dep) => {
                        debug_assert_ne!(new_dep, dep, "type resolution should move forward");
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
                let cache_key = match self.type_refs.get(&PgTypeRef::Oid(oid)) {
                    Some(ObjectRefState::Fetched(cache_key)) => cache_key,
                    _ => unreachable!("(BUG) Type resolution progressed but type is missing from the registry [oid={}]", oid),
                };
                let state = match self.type_resolutions.get_mut(cache_key) {
                    Some(s) => s,
                    _ => unreachable!(
                        "(BUG) Missing type resolution state for existing cache key {:?}",
                        cache_key
                    ),
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
        ty_ref: &PgTypeRef,
    ) -> Result<(&PgType<PgTypeOid>, ResolutionState), GetPgTypeError> {
        match self.type_refs.get(ty_ref) {
            None => Err(GetPgTypeError::Undeclared),
            Some(ObjectRefState::Declared) => Err(GetPgTypeError::Unfetched),
            Some(ObjectRefState::Missing) => Err(GetPgTypeError::Missing),
            Some(ObjectRefState::Fetched(cache_key)) => {
                let ty = match self.pg_type_cache.get(cache_key) {
                    None => unreachable!(
                        "(BUG) {:?} is fetched but the value is missing from the cache",
                        ty_ref
                    ),
                    Some(cached) => cached,
                };
                let resolution = match self.type_resolutions.get(cache_key) {
                    None => unreachable!(
                        "(BUG) {:?} is fetched but the resolution is missing from the catalog",
                        ty_ref
                    ),
                    Some(r) => *r,
                };
                Ok((ty, resolution))
            }
        }
    }

    /// Get a shallowly-resolved type from the local registry
    pub(crate) fn get_type(
        &self,
        ty_ref: &PgTypeRef,
    ) -> Result<&PgType<PgTypeOid>, GetPgTypeError> {
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
        match self.get_by_oid_with_resolution(&PgTypeRef::Oid(oid)) {
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
                        match registry.get_by_oid_with_resolution(&PgTypeRef::Oid(top)) {
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

impl PgTypeRef {
    pub(crate) fn as_oid(&self) -> Option<PgTypeOid> {
        match self {
            Self::Oid(oid) => Some(*oid),
            Self::Name(_) => None,
            Self::OidAndName(oid, _) => Some(*oid),
        }
    }
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
    use crate::ext::ustr::UStr;
    use crate::postgres::catalog::{
        FlagTypeAsMissingError, GetPgTypeError, LocalPgCatalog, PgLiveTypeRef, PgTypeRef,
        ResolvePgTypeError,
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
    fn test_flag_type_as_missing() {
        let mut registry = LocalPgCatalog::new();
        {
            let actual = registry.flag_type_as_missing(PgTypeRef::Name(UStr::Static("custom")));
            let expected = Ok(());
            assert_eq!(actual, expected);
        }
        {
            let actual = registry.flag_type_as_missing(PgTypeRef::Name(UStr::Static("custom")));
            let expected = Ok(());
            assert_eq!(actual, expected, "flagging as missing twice should succeed");
        }
        {
            let actual = registry.flag_type_as_missing(PgTypeRef::Name(UStr::Static("int4")));
            let expected = Err(FlagTypeAsMissingError::Conflict {
                ty_ref: PgTypeRef::Name(UStr::Static("int4")),
                old: PgBuiltinType::Int4.into_static_pg_type_with_oid().clone(),
            });
            assert_eq!(
                actual, expected,
                "conflicts with existing builtins should be detected"
            );
        }
        registry
            .insert_type(PgType {
                oid: PgTypeOid::from_u32(10000),
                name: "myint".into(),
                kind: PgTypeKind::Domain(PgBuiltinType::Int4.oid()),
            })
            .unwrap();
        {
            let actual = registry.flag_type_as_missing(PgTypeRef::Name(UStr::Static("myint")));
            let expected = Err(FlagTypeAsMissingError::Conflict {
                ty_ref: PgTypeRef::Name(UStr::Static("myint")),
                old: PgType {
                    oid: PgTypeOid::from_u32(10000),
                    name: "myint".into(),
                    kind: PgTypeKind::Domain(PgBuiltinType::Int4.oid()),
                },
            });
            assert_eq!(
                actual, expected,
                "conflicts with existing custom types should be detected"
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
