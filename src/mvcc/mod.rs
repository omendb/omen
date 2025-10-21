// Multi-Version Concurrency Control (MVCC) module
//
// Provides snapshot isolation for concurrent transactions through:
// - Transaction Oracle: timestamp allocation and lifecycle management
// - Versioned Storage: multi-version key-value storage
// - Schema Utilities: MVCC metadata column management

pub mod oracle;
pub mod schema;

pub use oracle::{
    CommittedTransaction, TransactionMode, TransactionOracle, TransactionState, TxnStatus,
};

pub use schema::{
    add_mvcc_columns, extract_user_schema, has_mvcc_columns, MvccIndices, MVCC_DELETED_COL,
    MVCC_TXN_ID_COL, MVCC_VERSION_COL,
};
