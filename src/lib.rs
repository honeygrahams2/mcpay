pub mod entrypoint;
pub mod instruction;
pub mod error;
pub mod processor;
pub mod state;

#[cfg(not(feature = "no-entrypoint"))]
use include_idl::include_idl;

#[cfg(not(feature = "no-entrypoint"))]
use solana_security_txt::security_txt;

#[cfg(not(feature = "no-entrypoint"))]
include_idl!();

#[cfg(not(feature = "no-entrypoint"))]
security_txt! {
    // Required fields
    name: "McPay",
    project_url: "https://mcswap.xyz",
    contacts: "email:nathan@airadlabs.com",
    policy: "https://github.com/McDegens-DAO/McSwap/blob/main/SECURITY.md"
}
