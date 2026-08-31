```toml
# =========================================================================
# Workspace Lint Configuration
# =========================================================================

[workspace.lints.rust]
# The basics of native compiler rigor
unsafe_code = "deny"                  # No unsafe blocks without explicit justification
missing_docs = "deny"                 # Requires documentation on everything public
missing_debug_implementations = "deny"# All public structs must implement Debug
unreachable_pub = "deny"              # Pub items that cannot be accessed publicly
unused_results = "deny"               # Forces handling of all returns (not just Results)
unused_qualifications = "deny"        # Avoids unnecessary absolute paths (e.g., std::vec::Vec)
trivial_casts = "deny"                # Forbids casts like `x as u32` if `x` is already `u32`
trivial_numeric_casts = "deny"        # Forbids useless numeric casts
unused_extern_crates = "deny"         # Cleans up dependency garbage

[workspace.lints.clippy]
# Level 1: The Main Groups
all = { level = "deny", priority = -1 }
pedantic = { level = "deny", priority = -1 }
nursery = { level = "warn", priority = -1 }

# Level 2: The "Give Me a Reason" Rule
# -------------------------------------------------------------------------
# 1. Forbids the use of #[allow()] generally.
allow_attributes = "deny"

# 2. IF, because of a macro, you need to make a local exception and use
# an #[allow()], you are OBLIGATED to write the `reason = "..."` attribute.
#
allow_attributes_without_reason = "deny"
# Example:
#
# #[allow(
#    clippy::allow_attributes,
#    clippy::pedantic,
#    reason = "async_trait macro allows underscore bindings internally. Forgiven."
# )]
# [async_trait]

# Level 3: Panic Prevention and Technical Debt (The Restriction group)
# -------------------------------------------------------------------------
unwrap_used = "deny"                  # Forbids .unwrap() (use pattern matching or the ? operator)
expect_used = "deny"                  # Forbids .expect() (propagate the error gracefully)
panic = "deny"                        # Forbids the explicit panic!() macro
todo = "deny"                         # Forbids the todo!() macro in production code
unimplemented = "deny"                # Forbids the unimplemented!() macro
unreachable = "deny"                  # Forces clarity in unreachable paths
fallible_impl_from = "deny"           # Forbids impl From that can fail (use TryFrom)
clone_on_ref_ptr = "deny"             # Forces Arc::clone(&x) instead of x.clone() for clarity
dbg_macro = "deny"                    # Forbids dbg!() forgotten in the code
print_stdout = "warn"                 # Restricts print!/println! (use the `tracing` or `log` crate)
use_self = "deny"                     # Forces the use of `Self` in implementations for clean refactoring
wildcard_dependencies = "deny"        # Forbids `crate = "*"` type dependencies in Cargo.toml
cargo = { level = "warn", priority = -1 }
multiple_crate_versions = "deny"      # Forbids having multiple versions of the same crate compiled

```
