use crate::Result;

pub fn inject(code: &str, deps: &[String]) -> Result<String> {
    let code = hotfix_nav_key(code);
    let deps = deps
        .iter()
        .map(|dep| format!("\"{}\"", js_escape(dep)))
        .collect::<Vec<_>>()
        .join(", ");
    Ok(format!(
        "{code}\n\n{}\n",
        HMR_TEMPLATE.replace("__DEPENDENCIES__", &deps)
    ))
}

fn hotfix_nav_key(code: &str) -> String {
    let needle = "function() { key.a(onUrlChange(_Browser_getUrl())); };";
    if code.contains("elm$browser$Browser$application") && code.contains(needle) {
        code.replace(needle, &format!("{needle}\n\tkey['elm-hot-nav-key'] = true;\n"))
    } else {
        code.to_string()
    }
}

fn js_escape(input: &str) -> String {
    input.replace('\\', "\\\\").replace('"', "\\\"")
}

const HMR_TEMPLATE: &str = r#"if (import.meta.hot) {
  const instances = import.meta.hot.data ? import.meta.hot.data.instances || {} : {}
  let uid = import.meta.hot.data ? import.meta.hot.data.uid || 0 : 0
  if (Object.keys(instances).length === 0) console.log("[vite-plugin-elm] HMR enabled")
  import.meta.hot.accept()
  import.meta.hot.accept([__DEPENDENCIES__], () => {
    console.log("[vite-plugin-elm] Dependency is updated")
  })
  import.meta.hot.on('hot-update-dependents', (data) => {
    console.log("[vite-plugin-elm] Request to hot update dependents: " + data.join(", "))
  })
  import.meta.hot.dispose((data) => {
    data.instances = instances
    data.uid = uid
  })
  const getId = () => ++uid
  const findPublicModules = (parent, path = "") => Object.keys(parent).flatMap((key) => {
    const child = parent[key]
    const currentPath = path ? path + "." + key : key
    return child && "init" in child ? [{ path: currentPath, module: child }] : findPublicModules(child || {}, currentPath)
  })
  findPublicModules(Elm).forEach(({ path, module }) => {
    const originalInit = module.init
    module.init = (args) => {
      const id = getId()
      instances[id] = {
        path,
        domNode: args && args.node ? args.node : document.body,
        flags: args && args.flags
      }
      return originalInit(args)
    }
  })
}"#;
