use color_eyre::eyre::{eyre, ContextCompat, Result};
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, ArrowFunctionExpression, CallExpression, Expression,
    ObjectPropertyKind, Program, Statement,
};
use tracing::instrument;

pub type ArrowFn<'a> = oxc_allocator::Box<'a, ArrowFunctionExpression<'a>>;

#[instrument(skip(program))]
fn get_root_element_name(program: &Program) -> Result<String> {
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
        return Err(eyre!(
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
        return Err(eyre!(
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
        return Err(eyre!(
            "Expected first argument of 'children: x.jsx()' call expression to be an Identifier"
        ));
    };

    Ok(root_element_name.into_string())
}

#[instrument(skip(program))]
fn get_root_element<'a>(program: &'a Program, root_element_name: &str) -> Result<&'a ArrowFn<'a>> {
    // const ...,
    //   Bm = () => {
    //        ^^^^^^^...
    //     const ...;
    //     return x.jsxs("main", {
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
    //   };
    //...^
    let root_element = program
        .body
        .iter()
        .find_map(|node| {
            if let Statement::VariableDeclaration(var_decl) = node {
                var_decl.declarations.iter().find_map(|decl| {
                    if let Some(Expression::ArrowFunctionExpression(arrow_func)) = &decl.init
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

    Ok(root_element)
}

#[instrument(skip(program))]
pub fn extract_root_element<'a>(program: &'a Program) -> Result<&'a ArrowFn<'a>> {
    let root_element_name = get_root_element_name(program)?;
    let root_element = get_root_element(program, &root_element_name)?;

    Ok(root_element)
}

pub type TopLevelElement<'a> = oxc_allocator::Box<'a, CallExpression<'a>>;

#[instrument(skip(root_element))]
fn get_all_top_level_elements<'a>(
    root_element: &'a ArrowFn<'a>,
) -> Result<Vec<&'a TopLevelElement<'a>>> {
    // const ...,
    //   Bm = () => {
    //     const ...;
    //     return x.jsxs("main", {
    //              ^^^^^^^^^^^^^^...
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
    //   };
    let main_jsxs_call = root_element
        .body
        .statements
        .iter()
        .find_map(|stmt| {
            if let Statement::ReturnStatement(ret_stmt) = stmt
                && let Some(Expression::CallExpression(call_expr)) = &ret_stmt.argument
                && call_expr.callee_name().is_some_and(|name| name == "jsxs")
            {
                Some(call_expr)
            } else {
                None
            }
        })
        .context("Failed to find x.jsxs() call in root element function body")?;

    //     return x.jsxs("main", {
    //                           ^...
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
            .context("No second argument found in () => x.jsxs(\"main\") call")?
    {
        obj_expr
    } else {
        return Err(eyre!(
            "Expected second argument of () => x.jsxs(\"main\") to be an ObjectExpression"
        ));
    };

    //     return x.jsxs("main", {
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
            "Failed to find 'children: []' array expression property in () => x.jsxs(\"main\") call props",
        )?;

    //       children: [
    //         x.jsx(Vp, {}),
    //           ^^^^^^^^^^^
    //         x.jsx(Bp, {}),
    //           ^^^^^^^^^^^...
    //         x.jsx($m, {}),
    //         x.jsx(Zp, {}),
    //         x.jsx(Up, {}),
    //         x.jsx(Hp, {}),
    //        ...^^^^^^^^^^^
    //         x.jsx(Qd, {}),
    //           ^^^^^^^^^^^
    //       ],
    let top_level_elements = children_prop_array
        .elements
        .iter()
        .filter_map(|elem| {
            if let ArrayExpressionElement::CallExpression(call_expr) = elem
                && let Some(name) = call_expr.callee_name()
                && (name == "jsx" || name == "jsxs")
            {
                Some(call_expr)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(top_level_elements)
}

pub struct TopLevelElements<'a> {
    pub platform_section: &'a TopLevelElement<'a>,
    pub reward_section: &'a TopLevelElement<'a>,
}

#[instrument(skip(root_element))]
pub fn extract_top_level_elements<'a>(
    root_element: &'a ArrowFn<'a>,
) -> Result<TopLevelElements<'a>> {
    let top_level_elements = get_all_top_level_elements(root_element)?;

    if top_level_elements.len() != 7 {
        return Err(eyre!(
            "Expected 7 top-level elements, found {}",
            top_level_elements.len()
        ));
    }

    let platform_section = top_level_elements
        .get(1)
        .context("Failed to find platform section top-level element")?;
    let reward_section = top_level_elements
        .get(2)
        .context("Failed to find reward section top-level element")?;

    Ok(TopLevelElements {
        platform_section,
        reward_section,
    })
}
