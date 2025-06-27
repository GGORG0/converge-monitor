use color_eyre::eyre::{ContextCompat, Result};
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, ArrowFunctionExpression, Expression, ObjectPropertyKind,
    Program, Statement,
};
use tracing::instrument;

#[instrument(skip(program))]
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

#[instrument(skip(program))]
pub fn get_top_level_element_names(
    program: &Program,
    root_element_name: &str,
) -> Result<Vec<String>> {
    // const ...,
    //   Wm = () =>
    //        ^^^^^...
    //     x.jsxs("main", {
    //       className: "flex flex-col w-full bg-[#fbfaf9]",
    //       "data-model-id": "3:27",
    //       children: [
    //         x.jsx(Vp, {}),
    //         x.jsx(Bp, {}),
    //         x.jsx($m, {}),
    //         x.jsx(Zp, {}),
    //         x.jsx(Up, {}),
    //         x.jsx(Hp, {}),
    //         x.jsx(Qd, {}),
    //       ],
    //     });
    let root_element = program
        .body
        .iter()
        .find_map(|node| {
            if let Statement::VariableDeclaration(var_decl) = node {
                var_decl.declarations.iter().find_map(|decl| {
                    if let Some(Expression::ArrowFunctionExpression(arrow_func)) = &decl.init
                        && arrow_func.expression
                        && let Some(name) = decl.id.get_identifier_name()
                        && name == root_element_name
                    {
                        Some(arrow_func)
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        })
        .context("Failed to find root element function in program")?;

    // const ...,
    //   Wm = () =>
    //     x.jsxs("main", {
    //       ^^^^^^^^^^^^^^...
    //       className: "flex flex-col w-full bg-[#fbfaf9]",
    //       "data-model-id": "3:27",
    //       children: [
    //         x.jsx(Vp, {}),
    //         x.jsx(Bp, {}),
    //         x.jsx($m, {}),
    //         x.jsx(Zp, {}),
    //         x.jsx(Up, {}),
    //         x.jsx(Hp, {}),
    //         x.jsx(Qd, {}),
    //       ],
    //     });
    //  ...^^
    let main_jsxs_call = if let Some(Statement::ExpressionStatement(expr_stmt)) =
        &root_element.body.statements.first()
        && let Expression::CallExpression(call_expr) = &expr_stmt.expression
    {
        call_expr
    } else {
        return Err(color_eyre::eyre::eyre!(
            "Expected root element function body to be a CallExpression"
        ));
    };

    // const ...,
    //   Wm = () =>
    //     x.jsxs("main", {
    //                    ^...
    //       className: "flex flex-col w-full bg-[#fbfaf9]",
    //       "data-model-id": "3:27",
    //       children: [
    //         x.jsx(Vp, {}),
    //         x.jsx(Bp, {}),
    //         x.jsx($m, {}),
    //         x.jsx(Zp, {}),
    //         x.jsx(Up, {}),
    //         x.jsx(Hp, {}),
    //         x.jsx(Qd, {}),
    //       ],
    //     });
    //  ...^
    let main_props = if let Argument::ObjectExpression(obj_expr) =
        main_jsxs_call
            .arguments
            .get(1)
            .context("No second argument found in () => x.jsxs() call")?
    {
        obj_expr
    } else {
        return Err(color_eyre::eyre::eyre!(
            "Expected second argument of () => x.jsxs() to be an ObjectExpression"
        ));
    };

    //     x.jsxs("main", {
    //       className: "flex flex-col w-full bg-[#fbfaf9]",
    //       "data-model-id": "3:27",
    //       children: [
    //                 ^...
    //         x.jsx(Vp, {}),
    //         x.jsx(Bp, {}),
    //         x.jsx($m, {}),
    //         x.jsx(Zp, {}),
    //         x.jsx(Up, {}),
    //         x.jsx(Hp, {}),
    //         x.jsx(Qd, {}),
    //       ],
    //    ...^
    //     });
    let children_prop_array = main_props
        .properties
        .iter()
        .find_map(|prop| {
            if let ObjectPropertyKind::ObjectProperty(obj_prop) = prop
                && obj_prop.key.name().is_some_and(|key| key == "children")
                && let Expression::ArrayExpression(arr_expr) = &obj_prop.value
            {
                Some(arr_expr)
            } else {
                None
            }
        })
        .context(
            "Failed to find 'children: []' array expression property in () => x.jsxs() call props",
        )?;

    //       children: [
    //         x.jsx(Vp, {}),
    //               ^^
    //         x.jsx(Bp, {}),
    //               ^^...
    //         x.jsx($m, {}),
    //         x.jsx(Zp, {}),
    //         x.jsx(Up, {}),
    //         x.jsx(Hp, {}),
    //            ...^^
    //         x.jsx(Qd, {}),
    //               ^^
    //       ],
    let top_level_elements = children_prop_array
        .elements
        .iter()
        .filter_map(|elem| {
            if let ArrayExpressionElement::CallExpression(call_expr) = elem
                && let Some(name) = call_expr.callee_name()
                && (name == "jsx" || name == "jsxs")
                && let Some(Argument::Identifier(ident)) = call_expr.arguments.first()
            {
                Some(ident.name)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(top_level_elements
        .into_iter()
        .map(|s| s.into_string())
        .collect())
}

#[instrument(skip(program))]
pub fn get_top_level_elements<'a>(
    program: &'a Program,
    top_level_element_names: &[String],
) -> Vec<&'a oxc_allocator::Box<'a, ArrowFunctionExpression<'a>>> {
    program
        .body
        .iter()
        .filter_map(|node| {
            if let Statement::VariableDeclaration(var_decl) = node {
                var_decl.declarations.iter().find_map(|decl| {
                    if let Some(Expression::ArrowFunctionExpression(arrow_func)) = &decl.init
                        && arrow_func.expression
                        && let Some(name) = decl.id.get_identifier_name()
                        && top_level_element_names.contains(&name.into_string())
                    {
                        Some(arrow_func)
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        })
        .collect()
}
