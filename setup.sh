#!/bin/bash

# Ensure we're in the repository root
cd "$(git rev-parse --show-toplevel)" || exit 1

# Create the pre-commit hook file
cat > .git/hooks/pre-commit << 'EOL'
#!/bin/bash
# Define color codes
BLUE='\033[0;34m'
NC='\033[0m' # No Color
# Ensure we're in the repository root
cd "$(git rev-parse --show-toplevel)" || exit 1
# Run rustfmt only on staged Rust files
echo -e "${BLUE}Running rustfmt...${NC}"
STAGED_RS_FILES=$(git diff --cached --name-only --diff-filter=ACM -- '*.rs')
if [ -n "$STAGED_RS_FILES" ]; then
    echo "Files to be formatted: $STAGED_RS_FILES"
    rustfmt --edition 2021 $STAGED_RS_FILES
    # Re-stage formatted files
    git add $STAGED_RS_FILES
else
    echo "No Rust files to format."
fi
# Run clippy
echo -e "${BLUE}Running clippy...${NC}"
cargo clippy -- -D warnings
# If clippy fails, prevent the commit
if [ $? -ne 0 ]; then
    echo "clippy found issues. Please fix them before committing."
    exit 1
fi
# Run cargo-shear
echo -e "${BLUE}Running cargo-shear...${NC}"
cargo shear
exit 0
EOL

# Make the pre-commit hook executable
chmod +x .git/hooks/pre-commit

echo "Pre-commit hook has been set up successfully!"