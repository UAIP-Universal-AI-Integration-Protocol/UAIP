#!/bin/bash
# API Documentation Generator for UAIP Hub
# Generates beautiful HTML documentation from OpenAPI spec

set -euo pipefail

# Configuration
API_SPEC="${API_SPEC:-docs/api/openapi.yaml}"
OUTPUT_DIR="${OUTPUT_DIR:-docs/api/generated}"
SERVE_PORT="${SERVE_PORT:-8080}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

# Check if required tools are available
check_tool() {
    local tool=$1
    local install_cmd=$2

    if ! command -v "$tool" &> /dev/null; then
        echo -e "${YELLOW}⚠ $tool not found${NC}"
        if [ -n "$install_cmd" ]; then
            echo -e "  Install with: ${BLUE}$install_cmd${NC}"
        fi
        return 1
    fi
    return 0
}

# Validate OpenAPI spec
validate_spec() {
    echo -e "${BLUE}Validating OpenAPI specification...${NC}"

    if [ ! -f "$API_SPEC" ]; then
        echo -e "${RED}✗ API spec not found: $API_SPEC${NC}"
        exit 1
    fi

    # Try to validate with openapi-generator if available
    if command -v openapi-generator &> /dev/null; then
        if openapi-generator validate -i "$API_SPEC" &> /dev/null; then
            echo -e "${GREEN}✓ OpenAPI spec is valid${NC}"
        else
            echo -e "${RED}✗ OpenAPI spec validation failed${NC}"
            openapi-generator validate -i "$API_SPEC"
            exit 1
        fi
    else
        echo -e "${YELLOW}⚠ openapi-generator not found, skipping validation${NC}"
    fi
}

# Generate Redoc documentation
generate_redoc() {
    echo -e "\n${BLUE}Generating Redoc documentation...${NC}"

    mkdir -p "$OUTPUT_DIR"

    # Create standalone HTML file with embedded spec
    cat > "$OUTPUT_DIR/redoc.html" <<'EOF'
<!DOCTYPE html>
<html>
  <head>
    <title>UAIP Hub API Documentation</title>
    <meta charset="utf-8"/>
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <link href="https://fonts.googleapis.com/css?family=Montserrat:300,400,700|Roboto:300,400,700" rel="stylesheet">
    <style>
      body {
        margin: 0;
        padding: 0;
      }
    </style>
  </head>
  <body>
    <redoc spec-url='openapi.yaml'></redoc>
    <script src="https://cdn.redoc.ly/redoc/latest/bundles/redoc.standalone.js"> </script>
  </body>
</html>
EOF

    # Copy OpenAPI spec to output directory
    cp "$API_SPEC" "$OUTPUT_DIR/openapi.yaml"

    echo -e "${GREEN}✓ Redoc documentation generated${NC}"
    echo -e "  Location: ${YELLOW}$OUTPUT_DIR/redoc.html${NC}"
}

# Generate Swagger UI documentation
generate_swagger_ui() {
    echo -e "\n${BLUE}Generating Swagger UI documentation...${NC}"

    mkdir -p "$OUTPUT_DIR"

    # Create Swagger UI HTML
    cat > "$OUTPUT_DIR/swagger-ui.html" <<'EOF'
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8">
    <title>UAIP Hub API - Swagger UI</title>
    <link rel="stylesheet" type="text/css" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css" />
    <link rel="icon" type="image/png" href="https://petstore.swagger.io/favicon-32x32.png" sizes="32x32" />
    <link rel="icon" type="image/png" href="https://petstore.swagger.io/favicon-16x16.png" sizes="16x16" />
    <style>
      html {
        box-sizing: border-box;
        overflow: -moz-scrollbars-vertical;
        overflow-y: scroll;
      }
      *, *:before, *:after {
        box-sizing: inherit;
      }
      body {
        margin:0;
        padding:0;
      }
    </style>
  </head>
  <body>
    <div id="swagger-ui"></div>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js" charset="UTF-8"> </script>
    <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-standalone-preset.js" charset="UTF-8"> </script>
    <script>
      window.onload = function() {
        window.ui = SwaggerUIBundle({
          url: "openapi.yaml",
          dom_id: '#swagger-ui',
          deepLinking: true,
          presets: [
            SwaggerUIBundle.presets.apis,
            SwaggerUIStandalonePreset
          ],
          plugins: [
            SwaggerUIBundle.plugins.DownloadUrl
          ],
          layout: "StandaloneLayout"
        });
      };
    </script>
  </body>
</html>
EOF

    echo -e "${GREEN}✓ Swagger UI documentation generated${NC}"
    echo -e "  Location: ${YELLOW}$OUTPUT_DIR/swagger-ui.html${NC}"
}

# Generate index page
generate_index() {
    echo -e "\n${BLUE}Generating documentation index...${NC}"

    cat > "$OUTPUT_DIR/index.html" <<'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>UAIP Hub API Documentation</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
            line-height: 1.6;
            color: #333;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            display: flex;
            align-items: center;
            justify-content: center;
        }
        .container {
            max-width: 800px;
            padding: 40px;
            background: white;
            border-radius: 10px;
            box-shadow: 0 10px 40px rgba(0,0,0,0.2);
        }
        h1 {
            color: #667eea;
            margin-bottom: 10px;
            font-size: 2.5em;
        }
        .subtitle {
            color: #666;
            margin-bottom: 30px;
            font-size: 1.2em;
        }
        .description {
            margin-bottom: 40px;
            line-height: 1.8;
            color: #555;
        }
        .card {
            border: 1px solid #e0e0e0;
            border-radius: 8px;
            padding: 20px;
            margin-bottom: 20px;
            transition: all 0.3s ease;
            cursor: pointer;
        }
        .card:hover {
            border-color: #667eea;
            box-shadow: 0 5px 15px rgba(102, 126, 234, 0.3);
            transform: translateY(-2px);
        }
        .card h3 {
            color: #667eea;
            margin-bottom: 10px;
            display: flex;
            align-items: center;
        }
        .card h3::before {
            content: "📖";
            margin-right: 10px;
            font-size: 1.5em;
        }
        .card p {
            color: #666;
            margin-bottom: 15px;
        }
        .button {
            display: inline-block;
            padding: 10px 20px;
            background: #667eea;
            color: white;
            text-decoration: none;
            border-radius: 5px;
            transition: background 0.3s ease;
        }
        .button:hover {
            background: #764ba2;
        }
        .features {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 15px;
            margin-bottom: 30px;
        }
        .feature {
            background: #f8f9fa;
            padding: 15px;
            border-radius: 5px;
            text-align: center;
        }
        .feature strong {
            color: #667eea;
            display: block;
            margin-bottom: 5px;
        }
    </style>
</head>
<body>
    <div class="container">
        <h1>UAIP Hub API</h1>
        <p class="subtitle">Universal AI Integration Protocol - API Documentation</p>

        <div class="description">
            <p>
                Welcome to the UAIP Hub API documentation. UAIP is a universal protocol that enables
                AI systems to discover, authenticate, control, and monitor physical IoT devices.
            </p>
        </div>

        <div class="features">
            <div class="feature">
                <strong>REST API</strong>
                <span>Full HTTP/JSON API</span>
            </div>
            <div class="feature">
                <strong>WebSocket</strong>
                <span>Real-time communication</span>
            </div>
            <div class="feature">
                <strong>Secure</strong>
                <span>JWT + X.509 + AES-256</span>
            </div>
            <div class="feature">
                <strong>Scalable</strong>
                <span>Production-ready</span>
            </div>
        </div>

        <div class="card" onclick="window.location.href='redoc.html'">
            <h3>Redoc Documentation</h3>
            <p>
                Beautiful, responsive API documentation with a three-panel design.
                Perfect for reading and exploring the API structure.
            </p>
            <a href="redoc.html" class="button">Open Redoc →</a>
        </div>

        <div class="card" onclick="window.location.href='swagger-ui.html'">
            <h3>Swagger UI</h3>
            <p>
                Interactive API documentation with "Try it out" functionality.
                Test API endpoints directly from your browser.
            </p>
            <a href="swagger-ui.html" class="button">Open Swagger UI →</a>
        </div>

        <div class="card" onclick="window.location.href='openapi.yaml'">
            <h3>OpenAPI Specification</h3>
            <p>
                Download the raw OpenAPI 3.0 specification file in YAML format.
                Use it to generate client libraries or import into tools.
            </p>
            <a href="openapi.yaml" class="button" download>Download YAML →</a>
        </div>
    </div>
</body>
</html>
EOF

    echo -e "${GREEN}✓ Documentation index generated${NC}"
    echo -e "  Location: ${YELLOW}$OUTPUT_DIR/index.html${NC}"
}

# Serve documentation locally
serve_docs() {
    echo -e "\n${BLUE}Starting documentation server...${NC}"

    if command -v python3 &> /dev/null; then
        echo -e "${GREEN}✓ Serving documentation at http://localhost:${SERVE_PORT}${NC}"
        echo -e "${YELLOW}Press Ctrl+C to stop${NC}\n"
        cd "$OUTPUT_DIR"
        python3 -m http.server "$SERVE_PORT"
    elif command -v python &> /dev/null; then
        echo -e "${GREEN}✓ Serving documentation at http://localhost:${SERVE_PORT}${NC}"
        echo -e "${YELLOW}Press Ctrl+C to stop${NC}\n"
        cd "$OUTPUT_DIR"
        python -m SimpleHTTPServer "$SERVE_PORT"
    else
        echo -e "${RED}✗ Python not found${NC}"
        echo -e "  Install Python or use another HTTP server to serve $OUTPUT_DIR"
        echo -e "  Example: cd $OUTPUT_DIR && npx http-server -p $SERVE_PORT"
        exit 1
    fi
}

# Main function
main() {
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BLUE}  UAIP Hub API Documentation Generator${NC}"
    echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

    # Validate spec
    validate_spec

    # Generate documentation
    generate_redoc
    generate_swagger_ui
    generate_index

    echo -e "\n${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}✓ Documentation generation complete!${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "\n${YELLOW}Generated files:${NC}"
    echo -e "  - ${BLUE}$OUTPUT_DIR/index.html${NC} (Documentation hub)"
    echo -e "  - ${BLUE}$OUTPUT_DIR/redoc.html${NC} (Redoc viewer)"
    echo -e "  - ${BLUE}$OUTPUT_DIR/swagger-ui.html${NC} (Swagger UI)"
    echo -e "  - ${BLUE}$OUTPUT_DIR/openapi.yaml${NC} (OpenAPI spec)"

    # Offer to serve documentation
    echo -e "\n${YELLOW}Would you like to serve the documentation locally? (y/n)${NC}"
    read -r serve_choice

    if [ "$serve_choice" = "y" ] || [ "$serve_choice" = "Y" ]; then
        serve_docs
    else
        echo -e "\n${BLUE}To serve documentation later, run:${NC}"
        echo -e "  cd $OUTPUT_DIR && python3 -m http.server $SERVE_PORT"
        echo -e "\n${GREEN}Done!${NC}"
    fi
}

# Run main function
main
