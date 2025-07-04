use std::collections::HashMap;

use color_eyre::eyre::{ContextCompat, Result, bail};
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, Expression, ObjectPropertyKind, Program, Statement,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;
use url::Url;

use crate::scraping::extract_data::{
    Item,
    top_level_elements::{ArrowFn, get_top_level_elements},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Platform {
    pub name: String,
    pub image: Url,
}

impl Item for Platform {
    fn name(&self) -> &str {
        &self.name
    }
}

#[instrument(skip(program, root_element))]
pub fn get_platforms<'a>(
    program: &'a Program,
    root_element: &'a ArrowFn<'a>,
) -> Result<Vec<Platform>> {
    let top_level_elements = get_top_level_elements(root_element)?;

    if top_level_elements.len() != 7 {
        bail!(
            "Expected 7 top-level elements, found {}",
            top_level_elements.len()
        );
    }

    //       children: [
    //         x.jsx(Vp, {}),
    //         x.jsx(Bp, {}),
    //           ^^^^^^^^^^^
    //         x.jsx($m, {}),
    //         x.jsx(Zp, {}),
    //         x.jsx(Up, {}),
    //         x.jsx(Hp, {}),
    //         x.jsx(Qd, {}),
    //       ],
    let top_level_element = top_level_elements
        .get(1)
        .context("Failed to find platform section top-level element")?;

    //         x.jsx(Bp, {}),
    //               ^^
    let top_level_element_name =
        if let Some(Argument::Identifier(ident)) = top_level_element.arguments.first() {
            ident.name
        } else {
            bail!("Expected first argument of top-level element to be an Identifier");
        };

    // const ...,
    //   Bp = () => {
    //        ^^^^^^^...
    //     const i = [
    //       {
    //         name: "SLACK",
    //         image: "https://c.animaapp.com/mc7vj4gxgq9MVb/img/image-2.png",
    //         width: "w-16",
    //         height: "h-16",
    //         leftPosition: "left-0",
    //       },
    //       {
    //         name: "DISCORD",
    //         image: "https://c.animaapp.com/mc7vj4gxgq9MVb/img/image-removebg-preview--1--1.png",
    //         width: "w-16",
    //         height: "h-16",
    //         leftPosition: "left-0.5",
    //       },
    //       {
    //         name: "SIGNAL",
    //         image: "https://c.animaapp.com/mc7vj4gxgq9MVb/img/image-3.png",
    //         width: "w-16",
    //         height: "h-16",
    //         leftPosition: "left-0 rounded-[10px]",
    //       },
    //       {
    //         name: "TELEGRAM",
    //         image: "https://c.animaapp.com/mc7vj4gxgq9MVb/img/image-4.png",
    //         width: "w-16",
    //         height: "h-16",
    //         leftPosition: "left-[7px]",
    //       },
    //     ];
    //     ...
    //   };
    //...^
    let platform_element = program
        .body
        .iter()
        .find_map(|node| {
            if let Statement::VariableDeclaration(var_decl) = node {
                var_decl.declarations.iter().find_map(|decl| {
                    if let Some(Expression::ArrowFunctionExpression(arrow_func)) = &decl.init
                        && let Some(name) = decl.id.get_identifier_name()
                        && name == top_level_element_name
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
        .context("Failed to find platform section element function in program")?;

    // const ...,
    //   Bp = () => {
    //     const i = [
    //               ^...
    //       {
    //         name: "SLACK",
    //         image: "https://c.animaapp.com/mc7vj4gxgq9MVb/img/image-2.png",
    //         width: "w-16",
    //         height: "h-16",
    //         leftPosition: "left-0",
    //       },
    //       {
    //         name: "DISCORD",
    //         image: "https://c.animaapp.com/mc7vj4gxgq9MVb/img/image-removebg-preview--1--1.png",
    //         width: "w-16",
    //         height: "h-16",
    //         leftPosition: "left-0.5",
    //       },
    //       {
    //         name: "SIGNAL",
    //         image: "https://c.animaapp.com/mc7vj4gxgq9MVb/img/image-3.png",
    //         width: "w-16",
    //         height: "h-16",
    //         leftPosition: "left-0 rounded-[10px]",
    //       },
    //       {
    //         name: "TELEGRAM",
    //         image: "https://c.animaapp.com/mc7vj4gxgq9MVb/img/image-4.png",
    //         width: "w-16",
    //         height: "h-16",
    //         leftPosition: "left-[7px]",
    //       },
    //     ];
    //  ...^
    //     ...
    //   };
    let platform_array = platform_element
        .body
        .statements
        .iter()
        .find_map(|node| {
            if let Statement::VariableDeclaration(var_decl) = node {
                var_decl.declarations.iter().find_map(|decl| {
                    if let Some(Expression::ArrayExpression(array_expr)) = &decl.init {
                        Some(array_expr)
                    } else {
                        None
                    }
                })
            } else {
                None
            }
        })
        .context("Failed to find platform array in platform section element function")?;

    let platforms = platform_array
        .elements
        .iter()
        .filter_map(|element| {
            if let ArrayExpressionElement::ObjectExpression(obj_expr) = element {
                let properties: HashMap<String, String> =
                    HashMap::from_iter(obj_expr.properties.iter().filter_map(|prop| {
                        if let ObjectPropertyKind::ObjectProperty(obj_prop) = prop
                            && let Expression::StringLiteral(str_lit) = &obj_prop.value
                            && let Some(name) = obj_prop.key.name()
                        {
                            Some((name.to_string(), str_lit.value.into_string()))
                        } else {
                            None
                        }
                    }));

                if let (Some(name), Some(image)) = (properties.get("name"), properties.get("image"))
                {
                    Some(Platform {
                        name: name.clone(),
                        image: image.parse().ok()?,
                    })
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(platforms)
}
