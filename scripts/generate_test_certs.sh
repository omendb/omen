#!/bin/bash
#
# Generate self-signed TLS certificates for testing OmenDB
#
# Usage: ./generate_test_certs.sh [output_dir]
#
# Generates:
# - server.key: Private key
# - server.crt: Self-signed certificate (valid for 365 days)
#
# WARNING: These are self-signed certificates for TESTING ONLY.
# For production, use certificates from a trusted Certificate Authority.

set -e

OUTPUT_DIR="${1:-.}"
mkdir -p "$OUTPUT_DIR"

echo "Generating self-signed TLS certificates..."
echo "Output directory: $OUTPUT_DIR"

# Generate private key
openssl genrsa -out "$OUTPUT_DIR/server.key" 2048
echo "✓ Generated private key: $OUTPUT_DIR/server.key"

# Generate self-signed certificate
openssl req -new -x509 \
    -key "$OUTPUT_DIR/server.key" \
    -out "$OUTPUT_DIR/server.crt" \
    -days 365 \
    -subj "/C=US/ST=State/L=City/O=OmenDB/OU=Dev/CN=localhost"

echo "✓ Generated certificate: $OUTPUT_DIR/server.crt"
echo ""
echo "Certificate details:"
openssl x509 -in "$OUTPUT_DIR/server.crt" -text -noout | grep -E "(Subject:|Issuer:|Not Before|Not After)"

echo ""
echo "✓ Done! Certificates generated in $OUTPUT_DIR"
echo ""
echo "Usage:"
echo "  export OMENDB_TLS_CERT=$OUTPUT_DIR/server.crt"
echo "  export OMENDB_TLS_KEY=$OUTPUT_DIR/server.key"
echo "  ./target/release/postgres_server --tls"
echo ""
echo "⚠️  WARNING: These are self-signed certificates for TESTING ONLY!"
echo "   Clients will need to use 'sslmode=require' and trust the self-signed cert."
