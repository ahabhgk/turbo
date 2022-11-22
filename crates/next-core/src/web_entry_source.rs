use anyhow::{anyhow, Result};
use turbo_tasks::{debug::ValueDebug, TryJoinIterExt, Value};
use turbo_tasks_env::ProcessEnvVc;
use turbo_tasks_fs::FileSystemPathVc;
use turbopack::ecmascript::EcmascriptModuleAssetVc;
use turbopack_core::{
    asset::AssetVc,
    chunk::{ChunkGroupVc, ChunkableAssetVc},
    resolve::{origin::PlainResolveOriginVc, parse::RequestVc},
};
use turbopack_dev_server::{
    html::DevHtmlAssetVc,
    source::{asset_graph::AssetGraphContentSourceVc, ContentSourceVc},
};

use crate::{
    embed_js::wrap_with_next_js_fs,
    next_client::context::{
        get_client_asset_context, get_client_chunking_context, get_client_runtime_entries,
        ContextType,
    },
};

#[turbo_tasks::function]
pub async fn create_web_entry_source(
    project_root: FileSystemPathVc,
    entry_requests: Vec<RequestVc>,
    server_root: FileSystemPathVc,
    env: ProcessEnvVc,
    eager_compile: bool,
    browserslist_query: &str,
) -> Result<ContentSourceVc> {
    // "turbo/demo"
    // let project_root = wrap_with_next_js_fs(project_root);

    let ty = Value::new(ContextType::Other);
    // make context: {
    //   environment,
    //   module_options_context,
    //   resolve_options_context,
    // },
    let context = get_client_asset_context(project_root, browserslist_query, ty);
    // seal context: {
    //   output_root_path: "",
    //   chunk_root_path: "_chunks",
    //   css_chunk_root_path: None,
    //   asset_root_path: "_assets",
    //   layer: None,
    //   enable_hot_module_replacement: true,
    // }
    let chunking_context = get_client_chunking_context(project_root, server_root, ty);
    let entries = get_client_runtime_entries(project_root, env, ty);

    // ["turbo/crates/next-core/js/src",
    // "[embedded_modules]/@vercel/turbopack-next/dev/bootstrap.ts"]
    let runtime_entries = entries.resolve_entries(context);

    // "turbo/demo/_"
    let origin = PlainResolveOriginVc::new(context, project_root.join("_")).as_resolve_origin();
    // ["src/index.jsx"]
    let entries = entry_requests
        .into_iter()
        .map(|request| async move {
            Ok(origin
                .resolve_asset(request, origin.resolve_options())
                .primary_assets()
                .await?
                .first()
                .copied())
        })
        .try_join()
        .await?;
    let chunks: Vec<_> = entries
        .into_iter()
        .flatten()
        .enumerate()
        .map(|(i, module)| async move {
            // module: SourceAsset {
            //   path: "src/index.jsx",
            //   context: {
            //     environment,
            //     module_options_context,
            //     resolve_options_context,
            //   },
            //   ty: Ecmascript,
            //   transforms: [StyledJsx, Emotion, React, PresetEnv, ...],
            //   environment,
            // }
            //
            // async xxxVc::resolve_from(super_trait_vc: impl Into<RawVc>) ->
            // Result<Option<Self>> specialization, super vc -> raw vc -> sub vc
            if let Some(ecmascript) = EcmascriptModuleAssetVc::resolve_from(module).await? {
                Ok(ecmascript
                    .as_evaluated_chunk(chunking_context, (i == 0).then_some(runtime_entries)))
            } else if let Some(chunkable) = ChunkableAssetVc::resolve_from(module).await? {
                // TODO this is missing runtime code, so it's probably broken and we should also
                // add an ecmascript chunk with the runtime code
                Ok(chunkable.as_chunk(chunking_context))
            } else {
                // TODO convert into a serve-able asset
                Err(anyhow!(
                    "Entry module is not chunkable, so it can't be used to bootstrap the \
                     application"
                ))
            }
        })
        .try_join()
        .await?;

    let entry_asset: AssetVc = DevHtmlAssetVc::new_with_body(
        // "FileSystemPath { fs: FileSystem(DevServerFileSystem /* virtual fs */), path:
        // "index.html", }"
        server_root.join("index.html"),
        // [ChunkGroup { entry: Chunk { /* as_asset() path: "_chunks/_0c3608.js" */ } }]
        chunks.into_iter().map(ChunkGroupVc::from_chunk).collect(),
        r#"<div id="root"></div>"#.to_string(),
    )
    .into();

    let graph: ContentSourceVc = if eager_compile {
        AssetGraphContentSourceVc::new_eager(server_root, entry_asset)
    } else {
        AssetGraphContentSourceVc::new_lazy(server_root, entry_asset)
    }
    .into();
    Ok(graph)
}
