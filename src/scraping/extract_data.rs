use color_eyre::eyre::{ContextCompat, Result};
use oxc_ast::ast::{Argument, Expression, ObjectPropertyKind, Program, Statement};

pub fn get_root_element_name(program: &Program) -> Result<String> {
    // Gd.createRoot(document.getElementById("app")).render(
    //   x.jsx(Y.StrictMode, { children: x.jsx(Wm, {}) })
    // );
    let create_root_call = program
        .body
        .iter()
        .find_map(|node| {
            if let Statement::ExpressionStatement(expr_stmt) = node
                && let Expression::CallExpression(call_expr) = &expr_stmt.expression
                && let Expression::StaticMemberExpression(static_member_expr) = &call_expr.callee
                && let Expression::CallExpression(inner_call_expr) = &static_member_expr.object
                && inner_call_expr
                    .callee_name()
                    .is_some_and(|name| name == "createRoot")
            {
                Some(call_expr)
            } else {
                None
            }
        })
        .context("Failed to find createRoot() call in program")?;

    //   x.jsx(Y.StrictMode, { children: x.jsx(Wm, {}) })
    //     ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
    let strict_mode_jsx_call = if let Argument::CallExpression(call_expr) = create_root_call
        .arguments
        .first()
        .context("No arguments found in createRoot() call")?
    {
        call_expr
    } else {
        return Err(color_eyre::eyre::eyre!(
            "Expected first (and only) argument of createRoot().render() to be a CallExpression"
        ));
    };

    //   x.jsx(Y.StrictMode, { children: x.jsx(Wm, {}) })
    //                       ^^^^^^^^^^^^^^^^^^^^^^^^^^^
    let strict_mode_props = if let Argument::ObjectExpression(obj_expr) = strict_mode_jsx_call
        .arguments
        .get(1)
        .context("No second argument found in createRoot().render(x.jsx()) call")?
    {
        obj_expr
    } else {
        return Err(color_eyre::eyre::eyre!(
            "Expected second argument of createRoot().render(x.jsx()) to be an ObjectExpression"
        ));
    };

    //   x.jsx(Y.StrictMode, { children: x.jsx(Wm, {}) })
    //                                   ^^^^^^^^^^^^^
    let children_prop_call = strict_mode_props.properties.iter().find_map(|prop| {
        if let ObjectPropertyKind::ObjectProperty(obj_prop) = prop
            && obj_prop.key.name().is_some_and(|key| key == "children")
            && let Expression::CallExpression(call_expr) = &obj_prop.value
        {
            Some(call_expr)
        } else {
            None
        }
    }).context("Failed to find 'children: x.jsx()' call expression property in createRoot().render(x.jsx()) call props")?;

    //   x.jsx(Y.StrictMode, { children: x.jsx(Wm, {}) })
    //                                         ^^
    let root_element_name = if let Argument::Identifier(ident) = children_prop_call
        .arguments
        .first()
        .context("No arguments found in 'children: x.jsx()' call expression")?
    {
        ident.name
    } else {
        return Err(color_eyre::eyre::eyre!(
            "Expected first argument of 'children: x.jsx()' call expression to be an Identifier"
        ));
    };

    Ok(root_element_name.into_string())
}
