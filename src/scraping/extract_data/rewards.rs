use std::collections::HashMap;

use color_eyre::eyre::Result;
use oxc_ast::ast::{ArrayExpressionElement, Expression, ObjectPropertyKind, Statement};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::scraping::extract_data::{Item, top_level_elements::ArrowFn};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reward {
    title: String,
    desccription: String,
    color: (u8, u8, u8),
    tokens: u8,
    icon: Option<char>,
}

impl Item for Reward {
    fn name(&self) -> &str {
        &self.title
    }
}

#[instrument(skip(root_element))]
pub fn get_rewards<'a>(root_element: &'a ArrowFn<'a>) -> Result<Vec<Reward>> {
    //   Bm = () => {
    //     const i = [
    //               ^...
    //         {
    //           title: "$5 HETZNER CREDITS",
    //           description:
    //             "HOST YOUR NEXT PROJECT WITH HETZNER (ONE OF THE BEST BUDGET SERVER PROVIDERS AROUND)",
    //           color: "bg-[#d83a2c]",
    //           tokens: 1,
    //           Icon: Hm,
    //         },
    //         {
    //           title: "PORKBUN CREDITS",
    //           description: "GET $10 OF PORKBUN CREDITS TO BUY DOMAINS",
    //           color: "bg-[#ff6b35]",
    //           tokens: 2,
    //           icon: "🐷",
    //         },
    //         ...
    //       ],
    //    ...^
    //       a = [
    //           ^...
    //         {
    //           title: "ANTHROPIC CREDITS",
    //           description: "GET $10 OF ANTHROPIC API CREDITS TO BUILD WITH CLAUDE!",
    //           color: "bg-[#d97706]",
    //           tokens: 2,
    //           Icon: Im,
    //         },
    //         {
    //           title: "$20 CLOUDFLARE CREDITS",
    //           description:
    //             "PERFECT FOR HOSTING WEBSITES (WITH WORKERS & PAGES), STORING IMAGES AND DATA (WITH R2 AND CF IMAGES), AND PROTECTING YOUR WEBSITES FROM DDoS ATTACKS.",
    //           color: "bg-[#f38020]",
    //           tokens: 4,
    //           Icon: Dm,
    //         },
    //         ...
    //       ];
    //    ...^
    //     ...
    //   };
    let reward_arrays = root_element
        .body
        .statements
        .iter()
        .flat_map(|node| {
            if let Statement::VariableDeclaration(var_decl) = node {
                var_decl
                    .declarations
                    .iter()
                    .filter_map(|decl| {
                        if let Some(Expression::ArrayExpression(array_expr)) = &decl.init {
                            Some(array_expr)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            } else {
                vec![]
            }
        })
        .collect::<Vec<_>>();

    let combined_reward_arrays = reward_arrays
        .iter()
        .flat_map(|array_expr| {
            array_expr
                .elements
                .iter()
                .filter_map(|element| {
                    if let ArrayExpressionElement::ObjectExpression(obj_expr) = element {
                        Some(obj_expr)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let platforms = combined_reward_arrays
        .iter()
        .filter_map(|obj_expr| {
            let properties: HashMap<String, String> =
                HashMap::from_iter(obj_expr.properties.iter().filter_map(|prop| {
                    if let ObjectPropertyKind::ObjectProperty(obj_prop) = prop
                        && let Some(name) = obj_prop.key.name()
                    {
                        match &obj_prop.value {
                            Expression::StringLiteral(str_lit) => Some(str_lit.value.into_string()),
                            Expression::NumericLiteral(num_lit) => Some(num_lit.value.to_string()), // TODO: converting f64 -> String -> u8 is meh
                            _ => None,
                        }
                        .map(|value| (name.to_string(), value))
                    } else {
                        None
                    }
                }));

            if let (Some(title), Some(description), Some(color), Some(tokens)) = (
                properties.get("title"),
                properties.get("description"),
                properties.get("color"),
                properties.get("tokens"),
            ) {
                let icon = properties.get("icon").and_then(|s| s.chars().next());

                let color = color.trim_start_matches("bg-[#").trim_end_matches(']');
                let color = u32::from_str_radix(color, 16).ok()?;
                let color = (
                    ((color >> 16) & 0xFF) as u8,
                    ((color >> 8) & 0xFF) as u8,
                    (color & 0xFF) as u8,
                );

                Some(Reward {
                    title: title.clone(),
                    desccription: description.clone(),
                    color,
                    tokens: tokens.parse().ok()?,
                    icon,
                })
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(platforms)
}
