#![cfg(feature = "test-sbf")]

use solana_program_test::{tokio, ProgramTest};

#[tokio::test]
async fn create() {
    let _context = ProgramTest::new("mpl_core_program", mpl_core::ID, None)
        .start_with_context()
        .await;
}
