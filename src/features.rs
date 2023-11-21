use crate::utils::{BuildConfig, log, LogLevel};

pub fn cfg_feat(build_config: &BuildConfig) -> (Vec<String>, Vec<String>) {
    log(LogLevel::Info, "Getting features...");
    let lib_features = vec!["fp_simd", "alloc", "multitask", "fs", "net", "fd", "pipe", "select", "epoll"];
    let mut features= build_config.features.clone();
    if features.iter().any(|feat| {
        feat == "fs" || feat == "net" || feat == "pipe" || feat == "select" || feat == "epoll"
    }) {
        features.push("fd".to_string());
    }
    
    let mut ax_feats = Vec::new();
    let mut lib_feats = Vec::new();
    //? Determine LOG and pci (Add environment variables later)
    ax_feats.push("log-level-warn".to_string());
    ax_feats.push("bus-pci".to_string());
    // get content of features
    for feat in features {
        if !lib_features.contains(&feat.as_str()) {
            ax_feats.push(feat);
        } else {
            lib_feats.push(feat);
        }
    }
    (ax_feats, lib_feats)
}

pub fn cfg_feat_addprefix(build_config: &BuildConfig) -> (Vec<String>, Vec<String>) {
    // Set prefix
    let ax_feat_prefix = "axfeat/";
    let lib_feat_prefix = "axlibc/";

    // Add prefix
    let (ax_feats_pre, lib_feats_pre) = cfg_feat(build_config);
    let ax_feats_final = ax_feats_pre.into_iter().map(|feat| format!("{}{}", ax_feat_prefix, feat)).collect::<Vec<String>>();
    let lib_feats_final = lib_feats_pre.into_iter().map(|feat| format!("{}{}", lib_feat_prefix, feat)).collect::<Vec<String>>();
    log(LogLevel::Debug, &format!("ax_feats_final : {:?}", ax_feats_final));
    log(LogLevel::Debug, &format!("lib_feats_final : {:?}", lib_feats_final));

    (ax_feats_final, lib_feats_final)
}
