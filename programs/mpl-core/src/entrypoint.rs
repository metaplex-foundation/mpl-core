use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult,
    program_error::PrintProgramError, pubkey::Pubkey,
};
use solana_security_txt::security_txt;

use crate::{error::MplCoreError, processor};

entrypoint!(process_instruction);

/// Entrypoint function
fn process_instruction<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    instruction_data: &[u8],
) -> ProgramResult {
    if let Err(error) = processor::process_instruction(program_id, accounts, instruction_data) {
        // catch the error so we can print it
        error.print::<MplCoreError>();
        return Err(error);
    }
    Ok(())
}

security_txt! {
    // Required fields
    name: "Mpl Core",
    project_url: "https://metaplex.com",
    contacts: "email:security@metaplex.foundation",
    policy: "Report suspected vulnerabilities privately by emailing security@metaplex.foundation before public disclosure.",

    // Optional fields
    preferred_languages: "en",
    source_code: "https://github.com/metaplex-foundation/mpl-core"
}
