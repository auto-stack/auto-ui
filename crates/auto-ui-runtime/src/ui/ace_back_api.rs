//! Bind declared backend functions to the checked widget HTTP lane.

use std::collections::BTreeSet;
use std::fmt::Write;
use std::path::{Path, PathBuf};

use ace_core::source::packages::PackageModuleInput;
use auto_lang_core::api::{ApiEndpoint, ApiExtractor, ResponseKind};

pub(super) const PACKAGE: &str = "back";
pub(super) const MODULE: &str = "api";
pub(super) const SCOPED_URL: &str = "auto:back:";

pub(super) fn append_contract(
    app_at: &Path,
    units: &mut Vec<PackageModuleInput>,
) -> Result<Option<PathBuf>, String> {
    let mut names = BTreeSet::new();
    for unit in units.iter() {
        if unit.source.contains("back.api") {
            let syntax = super::ace_route::package_syntax(&unit.source)?;
            for import in syntax.imports {
                if let [package, module, name] = import.path.as_slice() {
                    if package == PACKAGE
                        && module == MODULE
                        && (import.reexport || syntax.referenced.contains(import.bound_name()))
                    {
                        names.insert(name.clone());
                    }
                }
            }
        }
    }
    if names.is_empty() {
        return Ok(None);
    }
    let path = app_at
        .ancestors()
        .skip(1)
        .take(3)
        .find_map(auto_lang_core::config::resolve_back_api)
        .ok_or_else(|| "source.age.back-api: missing back/api.at".to_string())?;
    let error = |message: String| format!("source.age.back-api: {}: {message}", path.display());
    let source = std::fs::read_to_string(&path).map_err(|err| error(err.to_string()))?;
    let ast = auto_lang_core::Parser::from(source.as_str())
        .parse()
        .map_err(|err| error(err.to_string()))?;
    let api = ApiExtractor::new().extract(MODULE, &ast.stmts);
    let mut bindings = String::new();
    for name in names {
        let mut matches = api
            .endpoints
            .iter()
            .filter(|endpoint| endpoint.fn_name == name);
        let endpoint = matches
            .next()
            .ok_or_else(|| error(format!("undeclared endpoint `{name}`")))?;
        if matches.next().is_some() {
            return Err(error(format!("ambiguous endpoint `{name}`")));
        }
        write_binding(&mut bindings, endpoint).map_err(&error)?;
    }
    units.push(PackageModuleInput {
        package: PACKAGE.into(),
        version: "1.0.0".into(),
        id: MODULE.into(),
        source: bindings,
        provider: None,
    });
    Ok(Some(path))
}

fn write_binding(source: &mut String, endpoint: &ApiEndpoint) -> Result<(), String> {
    let name = &endpoint.fn_name;
    let unsupported = || format!("unsupported checked signature for `{name}`");
    let method = endpoint.method();
    let path = endpoint.path();
    let valid_path = path.strip_prefix("/api/").is_some_and(|path| {
        path.split('/').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        })
    });
    if !valid_path
        || !matches!(method.as_str(), "GET" | "POST")
        || endpoint.response != ResponseKind::Json
    {
        return Err(unsupported());
    }
    let mut params = Vec::new();
    let mut fields = Vec::new();
    for (index, param) in endpoint.params.iter().enumerate() {
        let ty = match (param.ty.as_str(), param.optional, param.default.is_none()) {
            ("str" | "String", false, true) => "str",
            ("int", false, true) => "int",
            ("bool", false, true) => "bool",
            _ => return Err(unsupported()),
        };
        params.push(format!("arg{index} {ty}"));
        fields.push(format!("{}: arg{index}", param.name));
    }
    let result = match endpoint.return_type.as_str() {
        "str" | "String" => "Str",
        "int" => "int",
        "bool" => "bool",
        "[]str" | "List<str>" | "List<String>" => "List<Str>",
        _ => return Err(unsupported()),
    };
    let params = params.join(", ");
    let fields = fields.join(", ");
    writeln!(source, "pub fn {name}({params}) {result} {{").unwrap();
    if fields.is_empty() {
        writeln!(source, "    let payload = \"{{}}\"").unwrap();
    } else {
        writeln!(source, "    let payload = json.stringify({{{fields}}})").unwrap();
    }
    if method == "GET" {
        writeln!(
            source,
            "    let result {result} = Http.get_json(\"{SCOPED_URL}{path}?\" + payload)"
        )
        .unwrap();
    } else {
        writeln!(
            source,
            "    let response = Http.post_json(\"{SCOPED_URL}{path}\", payload)"
        )
        .unwrap();
        writeln!(source, "    let result {result} = json.parse(response)").unwrap();
    }
    writeln!(source, "    return result\n}}").unwrap();
    Ok(())
}
