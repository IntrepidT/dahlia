#!/bin/bash

# Leptos 0.6 to 0.8 Migration Script
# Run this from your project root directory

set -e

echo "🚀 Starting Leptos 0.6 → 0.8 Migration..."

# Create backup
echo "📦 Creating backup..."
git add -A && git commit -m "Pre-migration backup (Leptos 0.6)" || echo "No changes to commit"

echo "🔧 Applying migration patterns..."

find src -name "*.rs" -type f -exec sed -i \
  \
  `# Signal API changes` \
  -e 's/use leptos::create_signal;/use leptos::prelude::*;/g' \
  -e 's/create_signal(/signal(/g' \
  -e 's/create_rw_signal(/RwSignal::new(/g' \
  -e 's/create_read_slice(/ReadSignal::derive(/g' \
  -e 's/create_write_slice(/WriteSignal::derive(/g' \
  \
  `# Effect API changes` \
  -e 's/create_effect(/Effect::new(/g' \
  -e 's/create_isomorphic_effect(/Effect::new(/g' \
  -e 's/create_render_effect(/RenderEffect::new(/g' \
  \
  `# Resource API changes` \
  -e 's/create_resource(/Resource::new(/g' \
  -e 's/create_local_resource(/LocalResource::new(/g' \
  -e 's/create_blocking_resource(/BlockingResource::new(/g' \
  \
  `# Memo API changes` \
  -e 's/create_memo(/Memo::new(/g' \
  \
  `# Action API changes` \
  -e 's/create_action(/Action::new(/g' \
  -e 's/create_server_action(/ServerAction::new(/g' \
  \
  `# Context API changes` \
  -e 's/provide_context(/context::provide(/g' \
  -e 's/use_context::/context::use_context::/g' \
  -e 's/expect_context(/context::expect_context(/g' \
  \
  `# Import changes` \
  -e 's/use leptos::\*;/use leptos::prelude::*;/g' \
  -e 's/use leptos_router::\*;/use leptos_router::components::*;\nuse leptos_router::hooks::*;/g' \
  -e 's/use leptos_meta::\*;/use leptos_meta::*;/g' \
  \
  `# View and component changes` \
  -e 's/\.into_view()/.into_any()/g' \
  -e 's/Fn() -> IntoView/Fn() -> impl IntoView/g' \
  -e 's/-> IntoView/-> impl IntoView/g' \
  \
  `# Router path changes` \
  -e 's/path="/path=path!("/g' \
  -e 's/path = "/path = path!("/g' \
  \
  `# Component macro changes` \
  -e 's/#\[component\]/#[component]/g' \
  \
  `# Server function changes` \
  -e 's/#\[server\]/#[server]/g' \
  -e 's/server_fn::/leptos::server_fn::/g' \
  \
  `# HTML attribute changes` \
  -e 's/class:/class=/g' \
  -e 's/style:/style=/g' \
  \
  `# Event handler changes` \
  -e 's/on:click=/on:click=/g' \
  -e 's/on:input=/on:input=/g' \
  -e 's/on:submit=/on:submit=/g' \
  \
  {} \;

echo "🔄 Applying router-specific changes..."

# Router-specific replacements
find src -name "*.rs" -type f -exec sed -i \
  \
  `# Router component imports` \
  -e 's/use leptos_router::{/use leptos_router::components::{/g' \
  -e 's/use leptos_router::Router;/use leptos_router::components::Router;/g' \
  -e 's/use leptos_router::Routes;/use leptos_router::components::Routes;/g' \
  -e 's/use leptos_router::Route;/use leptos_router::components::Route;/g' \
  -e 's/use leptos_router::A;/use leptos_router::components::A;/g' \
  \
  `# Router hook imports` \
  -e 's/use leptos_router::use_params;/use leptos_router::hooks::use_params;/g' \
  -e 's/use leptos_router::use_query;/use leptos_router::hooks::use_query;/g' \
  -e 's/use leptos_router::use_location;/use leptos_router::hooks::use_location;/g' \
  -e 's/use leptos_router::use_navigate;/use leptos_router::hooks::use_navigate;/g' \
  \
  `# Path macro usage` \
  -e 's/path="\([^"]*\)"/path=path!("\1")/g' \
  \
  {} \;

echo "📱 Applying meta tag changes..."

# Meta tag changes
find src -name "*.rs" -type f -exec sed -i \
  \
  `# Meta component changes` \
  -e 's/<Title>/<Title>/g' \
  -e 's/<Meta name=/<Meta name=/g' \
  -e 's/<Link rel=/<Link rel=/g' \
  \
  {} \;

echo "🎯 Applying server function changes..."

# Server function specific changes
find src -name "*.rs" -type f -exec sed -i \
  \
  `# Server function attribute changes` \
  -e 's/#\[server(.*)\]/#[server]/g' \
  \
  `# Server function error handling` \
  -e 's/ServerFnError/leptos::ServerFnError/g' \
  -e 's/server_fn::ServerFnError/leptos::ServerFnError/g' \
  \
  {} \;

echo "🔍 Fixing specific patterns..."

# Fix some common broken patterns that might need manual attention
find src -name "*.rs" -type f -exec sed -i \
  \
  `# Fix double parentheses from signal fixes` \
  -e 's/signal((/signal(/g' \
  \
  `# Fix path macro issues` \
  -e 's/path!(path!(/path!(/g' \
  \
  `# Fix import duplications` \
  -e '/use leptos::prelude::\*;/,+1 s/use leptos::\*;//g' \
  \
  {} \;

echo "🧹 Cleaning up Cargo.toml..."

# Update Cargo.toml dependencies (you may need to adjust versions)
if [ -f "Cargo.toml" ]; then
  sed -i \
    -e 's/leptos = { version = "0\.[67]/leptos = { version = "0.8/g' \
    -e 's/leptos_router = { version = "0\.[67]/leptos_router = { version = "0.8/g' \
    -e 's/leptos_meta = { version = "0\.[67]/leptos_meta = { version = "0.8/g' \
    -e 's/leptos_axum = { version = "0\.[67]/leptos_axum = { version = "0.8/g' \
    Cargo.toml
  echo "📦 Updated Cargo.toml dependencies"
fi

echo "✅ Migration patterns applied!"
echo ""
echo "🔍 Checking compilation..."
cargo check --message-format=short 2>&1 | head -20

echo ""
echo "📋 Next steps:"
echo "1. Review the compilation errors above"
echo "2. Check for any remaining manual fixes needed"
echo "3. Test your application thoroughly"
echo "4. Update any custom server function implementations"
echo "5. Review and update your router setup if needed"
echo ""
echo "⚠️  Common issues to watch for:"
echo "   - Match arms with different View types (add .into_any())"
echo "   - Custom component props that may need updating"
echo "   - Server function return types"
echo "   - Context providers and consumers"
echo ""
echo "🎉 Migration script complete!"
