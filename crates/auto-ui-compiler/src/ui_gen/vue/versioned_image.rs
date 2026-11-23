use std::fmt::Write;

pub enum VueScriptLanguage {
    JavaScript,
    TypeScript,
}

/// Share the resource loading and disposal contract between both Vue emitters.
pub fn append_versioned_image_directive(script: &mut String, language: VueScriptLanguage) {
    script.push('\n');
    let (map_types, element_type, binding_type, entry_type) = match language {
        VueScriptLanguage::JavaScript => ("", "", "", ""),
        VueScriptLanguage::TypeScript => {
            script.push_str(
                "type VersionedImage = {\n  source: string\n  identity: string\n  request: AbortController\n  objectUrl?: string\n}\n",
            );
            (
                "<HTMLImageElement, VersionedImage>",
                ": HTMLImageElement",
                ": { value: [string, string] }",
                ": VersionedImage",
            )
        }
    };
    writeln!(script, "const versionedImages = new WeakMap{map_types}()").unwrap();
    writeln!(
        script,
        "function releaseVersionedImage(element{element_type}) {{"
    )
    .unwrap();
    script.push_str(
        r#"  const previous = versionedImages.get(element)
  if (!previous) return
  previous.request.abort()
  if (previous.objectUrl) URL.revokeObjectURL(previous.objectUrl)
  versionedImages.delete(element)
}
"#,
    );
    writeln!(
        script,
        "async function updateVersionedImage(element{element_type}, binding{binding_type}) {{"
    )
    .unwrap();
    script.push_str(
        r#"  const [source, identity] = binding.value
  const previous = versionedImages.get(element)
  if (previous?.source === source && previous.identity === identity) return
  releaseVersionedImage(element)
  element.removeAttribute('src')
  if (!source) return
"#,
    );
    writeln!(
        script,
        "  const current{entry_type} = {{ source, identity, request: new AbortController() }}"
    )
    .unwrap();
    script.push_str(
        r#"  versionedImages.set(element, current)
  try {
    const url = new URL(source, document.baseURI)
    if (url.protocol !== 'http:' && url.protocol !== 'https:') {
      element.src = source
      return
    }
    const request = () => fetch(url, {
      cache: 'reload', credentials: 'include', mode: 'no-cors', signal: current.request.signal,
    })
    const workerTicket = url.origin === location.origin
      && /\/api\/__auto\/media\/[a-f0-9]{32}\/\d+$/.test(url.pathname)
    let response = await request()
    for (let attempt = 0; workerTicket && response.status === 503 && attempt < 8; attempt++) {
      // The media worker can publish the rendition after it issues the URI.
      await response.body?.cancel()
      await new Promise(resolve => setTimeout(resolve, Math.min(500, 125 * 2 ** attempt)))
      if (versionedImages.get(element) !== current) return
      response = await request()
    }
    if (versionedImages.get(element) !== current) return
    if (response.type === 'opaque') {
      // The reload refreshes the HTTP cache even for images without CORS.
      // A fragment changes Blink's decoded-image key without changing the request URI.
      if (!url.hash) url.hash = 'auto-ui-' + encodeURIComponent(identity)
      element.src = url.href
      return
    }
    if (!response.ok) throw Error('Image request failed')
    const blob = await response.blob()
    if (versionedImages.get(element) !== current) return
    current.objectUrl = URL.createObjectURL(blob)
    element.src = current.objectUrl + url.hash
  } catch {
    if (versionedImages.get(element) === current) element.dispatchEvent(new Event('error'))
  }
}
const vAutoVersionedImage = {
  mounted: updateVersionedImage,
  updated: updateVersionedImage,
  unmounted: releaseVersionedImage,
}
"#,
    );
}
