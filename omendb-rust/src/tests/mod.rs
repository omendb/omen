//! Comprehensive test suite for OmenDB
//! Target: 80% code coverage for production readiness

#[cfg(test)]
mod index_tests;

#[cfg(test)]
mod storage_tests;

#[cfg(test)]
mod integration_tests;

#[cfg(test)]
mod stress_tests;