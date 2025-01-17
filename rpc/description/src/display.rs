// Copyright (c) 2024 RBB S.r.l
// opensource@mintlayer.org
// SPDX-License-Identifier: MIT
// Licensed under the MIT License;
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// https://github.com/mintlayer/mintlayer-core/blob/master/LICENSE
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::{Interface, Method, MethodKindData, Module, RpcDocs, ValueHint};

impl std::fmt::Display for RpcDocs<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            title,
            version,
            description,
            interface,
        } = self;

        write!(f, "# RPC documentation for {title}\n\n")?;
        write!(f, "Version `{version}`.\n\n")?;
        if !description.is_empty() {
            write!(f, "{description}\n\n")?;
        }
        write!(f, "{interface}")?;

        Ok(())
    }
}

impl std::fmt::Display for Interface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self { modules } = self;

        for module in modules {
            module.fmt(f)?;
        }

        Ok(())
    }
}

impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            name,
            description,
            methods,
        } = self;

        writeln!(f, "## Module `{name}`\n")?;
        if !description.trim().is_empty() {
            writeln!(f, "{description}\n")?;
        }
        for method in *methods {
            method.fmt(f)?;
        }

        Ok(())
    }
}

fn code_block<T: std::fmt::Display>(
    header: &str,
    content: T,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{header}:")?;
    writeln!(f, "```")?;
    writeln!(f, "{content}")?;
    writeln!(f, "```\n")?;
    Ok(())
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            name,
            description,
            params,
            kind_data,
        } = self;

        let kind = match kind_data {
            MethodKindData::Subscription { .. } => "Subscription",
            MethodKindData::Method { .. } => "Method",
        };

        writeln!(f, "### {kind} `{name}`\n")?;
        if !description.trim().is_empty() {
            writeln!(f, "{description}\n")?;
        }
        code_block("Parameters", params, f)?;

        match kind_data {
            MethodKindData::Method { return_type } => {
                code_block("Returns", return_type, f)?;
            }
            MethodKindData::Subscription {
                unsubscribe_name,
                item_type,
            } => {
                code_block("Produces", item_type, f)?;
                writeln!(f, "Unsubscribe using `{unsubscribe_name}`.\n")?;
                writeln!(f, "Note: Subscriptions only work over WebSockets.\n")?;
            }
        }

        Ok(())
    }
}

impl ValueHint {
    fn collect_choices<'a>(start: &'a [&'a Self]) -> Vec<&'a Self> {
        let mut out = Vec::new();
        Self::collect_choices_impl(&mut out, start);
        out
    }

    fn collect_choices_impl<'a>(out: &mut Vec<&'a Self>, hints: &'a [&'a Self]) {
        for hint in hints {
            match *hint {
                Self::Choice(subhints) => Self::collect_choices_impl(out, subhints),
                hint => out.push(hint),
            }
        }
    }

    fn fmt_indent(&self, indent: usize, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let indent_str = "    ";
        let indent_len = indent_str.len();
        let next_indent = indent + indent_len;

        // Implementing indentation can be a bit tricky. Here are some general rules:
        //
        // * Use `indent_str` to indent something beyond the current indentation level.
        // * Use `next_indent` in recursive calls for nested structures.
        // * Emit the correct number of spaces according to the current indentation level after
        //   each newline character (and nowhere else). Use `write!(f, "\n{:indent$}", "")?`.

        match self {
            ValueHint::Prim(h) => f.write_str(h)?,

            ValueHint::StrLit(s) => write!(f, "{s:?}")?,

            ValueHint::Choice(hints) => match Self::collect_choices(hints).as_slice() {
                [] => f.write_str("impossible")?,
                [hint] => hint.fmt_indent(indent, f)?,
                hints => {
                    write!(f, "EITHER OF\n{:indent$}", "")?;
                    let pad = 2 * indent_len - 2;
                    for (n, hint) in hints.iter().enumerate() {
                        let n = n + 1;
                        write!(f, "{n:pad$}) ")?;
                        hint.fmt_indent(next_indent + indent_len, f)?;
                        if n < hints.len() {
                            write!(f, "\n{:indent$}", "")?;
                        }
                    }
                }
            },

            ValueHint::Object(hints) => match *hints {
                [] => f.write_str("{}")?,
                [(name, hint)] => {
                    write!(f, "{{ {name:?}: ")?;
                    hint.fmt_indent(indent, f)?;
                    f.write_str(" }")?;
                }
                hints => {
                    write!(f, "{{\n{:indent$}", "")?;
                    for (name, hint) in hints {
                        write!(f, "{indent_str}{name:?}: ")?;
                        hint.fmt_indent(next_indent, f)?;
                        write!(f, ",\n{:indent$}", "")?;
                    }
                    f.write_str("}")?;
                }
            },

            ValueHint::Map(key, val) => {
                f.write_str("{ ")?;
                key.fmt_indent(indent, f)?;
                f.write_str(": ")?;
                val.fmt_indent(indent, f)?;
                f.write_str(", .. }")?;
            }

            ValueHint::Tuple(hints) => match *hints {
                [] => f.write_str("[]")?,
                [hint] => {
                    f.write_str(" [")?;
                    hint.fmt_indent(indent, f)?;
                    f.write_str(" ]")?;
                }
                hints => {
                    write!(f, "[\n{:indent$}", "")?;
                    for hint in hints {
                        f.write_str(indent_str)?;
                        hint.fmt_indent(next_indent, f)?;
                        write!(f, ",\n{:indent$}", "")?;
                    }
                    f.write_str("]")?;
                }
            },

            ValueHint::Array(inner) => {
                f.write_str("[ ")?;
                inner.fmt_indent(indent, f)?;
                f.write_str(", .. ]")?;
            }
        }

        Ok(())
    }
}

impl std::fmt::Display for ValueHint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_indent(0, f)
    }
}
