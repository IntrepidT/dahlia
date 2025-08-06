#!/bin/bash

# Leptos 0.8 Specific Issue Fixes
# Run these individually based on your specific errors

echo "🎯 Leptos 0.8 Specific Fixes"

# Function 1: Fix View Type Mismatches
fix_view_types() {
    echo "🔧 Fixing View type mismatches..."
    
    find src -name "*.rs" -exec sed -i \
        -e 's/\.into_view()/.into_any()/g' \
        -e 's/-> IntoView/-> impl IntoView/g' \
        -e 's/Fn() -> IntoView/Fn() -> impl IntoView/g' \
        -e 's/FnOnce() -> IntoView/FnOnce() -> impl IntoView/g' \
        {} \;
    
    echo "✅ View types fixed"
}

# Function 2: Fix Signal API Changes
fix_signals() {
    echo "🔧 Fixing Signal API..."
    
    find src -name "*.rs" -exec sed -i \
        -e 's/create_signal(/signal(/g' \
        -e 's/create_rw_signal(/RwSignal::new(/g' \
        -e 's/create_memo(/Memo::new(/g' \
        -e 's/\.get()/.get()/g' \
        -e 's/\.set(/\.set(/g' \
        -e 's/\.update(/\.update(/g' \
        {} \;
    
    echo "✅ Signals fixed"
}

# Function 3: Fix Effect API Changes  
fix_effects() {
    echo "🔧 Fixing Effect API..."
    
    find src -name "*.rs" -exec sed -i \
        -e 's/create_effect(/Effect::new(/g' \
        -e 's/create_effect(move |_|/Effect::new(move |_|/g' \
        -e 's/create_isomorphic_effect(/Effect::new(/g' \
        -e 's/create_render_effect(/RenderEffect::new(/g' \
        {} \;
    
    echo "✅ Effects fixed"
}

# Function 4: Fix Resource API Changes
fix_resources() {
    echo "🔧 Fixing Resource API..."
    
    find src -name "*.rs" -exec sed -i \
        -e 's/create_resource(/Resource::new(/g' \
        -e 's/create_local_resource(/LocalResource::new(/g' \
        -e 's/create_blocking_resource(/BlockingResource::new(/g' \
        {} \;
    
    echo "✅ Resources fixed"
}

# Function 5: Fix Router Changes
fix_router() {
    echo "🔧 Fixing Router API..."
    
    find src -name "*.rs" -exec sed -i \
        -e 's/path="\([^"]*\)"/path=path!("\1")/g' \
        -e 's/use leptos_router::\*;/use leptos_router::components::*;\nuse leptos_router::hooks::*;/g' \
        -e 's/use leptos_router::{Router, Routes, Route}/use leptos_router::components::{Router, Routes, Route}/g' \
        {} \;
    
    # Add path import if not present
    find src -name "*.rs" -exec grep -l "path!" {} \; | \
    xargs -I {} sed -i '1i use leptos_router::path;' {}
    
    echo "✅ Router fixed"
}

# Function 6: Fix Action API Changes
fix_actions() {
    echo "🔧 Fixing Action API..."
    
    find src -name "*.rs" -exec sed -i \
        -e 's/create_action(/Action::new(/g' \
        -e 's/create_server_action(/ServerAction::new(/g' \
        -e 's/create_multi_action(/MultiAction::new(/g' \
        {} \;
    
    echo "✅ Actions fixed"
}

# Function 7: Fix Import Statements
fix_imports() {
    echo "🔧 Fixing Import statements..."
    
    find src -name "*.rs" -exec sed -i \
        -e 's/use leptos::\*;/use leptos::prelude::*;/g' \
        -e 's/use leptos::{.*};/use leptos::prelude::*;/g' \
        -e '1i use leptos::prelude::*;' \
        {} \;
    
    # Remove duplicate imports
    find src -name "*.rs" -exec awk '!seen[$0]++' {} \; -exec mv awk_output {} \;
    
    echo "✅ Imports fixed"
}

# Function 8: Fix Server Functions
fix_server_functions() {
    echo "🔧 Fixing Server Functions..."
    
    find src -name "*.rs" -exec sed -i \
        -e 's/#\[server\]/#[server]/g' \
        -e 's/ServerFnError/leptos::ServerFnError/g' \
        -e 's/server_fn::/leptos::server_fn::/g' \
        {} \;
    
    echo "✅ Server functions fixed"
}

# Function 9: Fix Context API
fix_context() {
    echo "🔧 Fixing Context API..."
    
    find src -name "*.rs" -exec sed -i \
        -e 's/provide_context(/context::provide(/g' \
        -e 's/use_context::/context::use_context::/g' \
        -e 's/expect_context(/context::expect_context(/g' \
        {} \;
    
    echo "✅ Context API fixed"
}

# Function 10: Fix Component Attributes
fix_component_attributes() {
    echo "🔧 Fixing Component attributes..."
    
    find src -name "*.rs" -exec sed -i \
        -e 's/class:/class=/g' \
        -e 's/style:/style=/g' \
        -e 's/id:/id=/g' \
        {} \;
    
    echo "✅ Component attributes fixed"
}

# Main execution
case "${1:-all}" in
    "views"|"view")
        fix_view_types
        ;;
    "signals"|"signal")
        fix_signals
        ;;
    "effects"|"effect")
        fix_effects
        ;;
    "resources"|"resource")
        fix_resources
        ;;
    "router")
        fix_router
        ;;
    "actions"|"action")
        fix_actions
        ;;
    "imports"|"import")
        fix_imports
        ;;
    "server")
        fix_server_functions
        ;;
    "context")
        fix_context
        ;;
    "attributes"|"attr")
        fix_component_attributes
        ;;
    "all")
        echo "🚀 Running all fixes..."
        fix_imports
        fix_signals
        fix_effects
        fix_resources
        fix_actions
        fix_context
        fix_router
        fix_server_functions
        fix_view_types
        fix_component_attributes
        echo "🎉 All fixes applied!"
        ;;
    *)
        echo "Usage: $0 [views|signals|effects|resources|router|actions|imports|server|context|attributes|all]"
        echo ""
        echo "Available fixes:"
        echo "  views      - Fix IntoView and view type issues"
        echo "  signals    - Fix create_signal -> signal changes"
        echo "  effects    - Fix create_effect -> Effect::new changes"  
        echo "  resources  - Fix create_resource -> Resource::new changes"
        echo "  router     - Fix router path and import changes"
        echo "  actions    - Fix create_action -> Action::new changes"
        echo "  imports    - Fix import statements"
        echo "  server     - Fix server function changes"
        echo "  context    - Fix context API changes"
        echo "  attributes - Fix component attribute syntax"
        echo "  all        - Apply all fixes (default)"
        exit 1
        ;;
esac

echo ""
echo "🔍 Running cargo check..."
cargo check --message-format=short 2>&1 | head -10
