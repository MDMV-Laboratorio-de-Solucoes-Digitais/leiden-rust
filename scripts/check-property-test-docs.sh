#!/usr/bin/env bash
# Checks that all property_tests modules have doc comments linking to invariants.
# Enforces SC-007: each #[cfg(test)] mod property_tests MUST have a
# /// Verifies INV-XXX doc comment.
#
# Also enforces FR-006 topology coverage: each property_tests module that
# references graph generators MUST use at least 3 distinct topologies from
# {ErdosRenyi, StochasticBlock, ScaleFree, ParallelEdges, DisconnectedGraph}.

set -euo pipefail

# Find all Rust source files
FILES=$(find crates -name "*.rs" -type f 2>/dev/null || true)

if [ -z "$FILES" ]; then
    echo "No Rust files found."
    exit 0
fi

ERRORS=0

# Check 1: property_tests modules must have doc comments
while IFS= read -r file; do
    # Check if file contains property_tests module definitions
    if grep -q 'mod property_tests' "$file"; then
        # Extract line numbers of mod property_tests declarations
        while IFS= read -r line_num; do
            # Check the line above for doc comment
            doc_line=$((line_num - 1))
            doc_comment=$(sed -n "${doc_line}p" "$file" 2>/dev/null || true)

            if [[ ! "$doc_comment" =~ Verifies\ INV- ]]; then
                echo "ERROR:$file:$doc_line: property_tests module at line $line_num missing '/// Verifies INV-XXX' doc comment"
                ERRORS=$((ERRORS + 1))
            fi
        done < <(grep -n 'mod property_tests' "$file" | cut -d: -f1)
    fi
done <<< "$FILES"

# Check 2: Topology coverage (FR-006)
# Only applies to modules that reference at least 1 topology
TOPOLOGIES=("ErdosRenyi" "StochasticBlock" "ScaleFree" "ParallelEdges" "DisconnectedGraph")

while IFS= read -r file; do
    # Check if file contains property_tests module definitions
    if grep -q 'mod property_tests' "$file"; then
        # Count distinct topologies referenced
        found_topologies=0
        for topology in "${TOPOLOGIES[@]}"; do
            if grep -q "$topology" "$file"; then
                found_topologies=$((found_topologies + 1))
            fi
        done

        # If at least 1 topology is referenced, require at least 3
        if [ "$found_topologies" -ge 1 ] && [ "$found_topologies" -lt 3 ]; then
            echo "ERROR:$file: property_tests module uses only $found_topologies topology(s); FR-006 requires at least 3 distinct topologies"
            ERRORS=$((ERRORS + 1))
        fi
    fi
done <<< "$FILES"

if [ "$ERRORS" -gt 0 ]; then
    echo "FAILED: $ERRORS error(s) found"
    exit 1
fi

echo "PASSED: All property_tests modules comply with SC-007 and FR-006"
exit 0
