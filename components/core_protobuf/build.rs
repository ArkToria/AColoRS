const STRUCT_PATHS: &[&str] = &[
    "v2ray.config.LogObject",
    "v2ray.config.APIObject",
    "v2ray.config.DNSObject",
    "v2ray.config.RoutingObject",
    "v2ray.config.RoutingObject.RuleObject",
    "v2ray.config.RoutingObject.BalancerObject",
    "v2ray.config.RoutingObject.BalancerObject.StrategyObject",
    "v2ray.config.PolicyObject",
    "v2ray.config.PolicyObject.LevelPolicyObject",
    "v2ray.config.PolicyObject.SystemPolicyObject",
    "v2ray.config.StreamSettingsObject",
    "v2ray.config.StreamSettingsObject.TLSObject",
    "v2ray.config.StreamSettingsObject.SockoptObject",
    "v2ray.config.SOCKSObject",
    "v2ray.config.SOCKSObject.AccountObject",
    "v2ray.config.SOCKSObject.InboundConfigurationObject",
    "v2ray.config.SOCKSObject.UserObject",
    "v2ray.config.SOCKSObject.ServerObject",
    "v2ray.config.SOCKSObject.OutboundConfigurationObject",
    "v2ray.config.HTTPObject",
    "v2ray.config.HTTPObject.AccountObject",
    "v2ray.config.HTTPObject.InboundConfigurationObject",
    "v2ray.config.HTTPObject.OutboundConfigurationObject",
    "v2ray.config.TrojanObject",
    "v2ray.config.TrojanObject.ClientObject",
    "v2ray.config.TrojanObject.FallbackObject",
    "v2ray.config.TrojanObject.InboundConfigurationObject",
    "v2ray.config.TrojanObject.ServerObject",
    "v2ray.config.TrojanObject.OutboundConfigurationObject",
    "v2ray.config.ShadowsocksObject",
    "v2ray.config.ShadowsocksObject.InboundConfigurationObject",
    "v2ray.config.ShadowsocksObject.ServerObject",
    "v2ray.config.ShadowsocksObject.OutboundConfigurationObject",
    "v2ray.config.VMESSObject",
    "v2ray.config.VMESSObject.ClientObject",
    "v2ray.config.VMESSObject.DefaultObject",
    "v2ray.config.VMESSObject.DetourObject",
    "v2ray.config.VMESSObject.InboundConfigurationObject",
    "v2ray.config.VMESSObject.UserObject",
    "v2ray.config.VMESSObject.ServerObject",
    "v2ray.config.VMESSObject.OutboundConfigurationObject",
    "v2ray.config.DokodemoDoorObject",
    "v2ray.config.DokodemoDoorObject.InboundConfigurationObject",
    "v2ray.config.InboundObject",
    "v2ray.config.InboundObject.InboundSettings",
    "v2ray.config.InboundObject.SniffingObject",
    "v2ray.config.InboundObject.AllocateObject",
    "v2ray.config.OutboundObject",
    "v2ray.config.OutboundObject.OutboundSettings",
    "v2ray.config.OutboundObject.ProxySettingsObject",
    "v2ray.config.OutboundObject.MuxObject",
    "v2ray.config.TransportObject",
    "v2ray.config.TransportObject.Headers",
    "v2ray.config.TransportObject.TCPObject",
    "v2ray.config.TransportObject.TCPObject.HeaderObject",
    "v2ray.config.TransportObject.TCPObject.HeaderObject.HTTPRequestObject",
    "v2ray.config.TransportObject.TCPObject.HeaderObject.HTTPResponseObject",
    "v2ray.config.TransportObject.KCPObject.HeaderObject",
    "v2ray.config.TransportObject.WebSocketObject",
    "v2ray.config.TransportObject.HTTPObject",
    "v2ray.config.TransportObject.QUICObject",
    "v2ray.config.TransportObject.QUICObject.HeaderObject",
    "v2ray.config.TransportObject.DomainSocketObject",
    "v2ray.config.TransportObject.GRPCObject",
    "v2ray.config.StatsObject",
    "v2ray.config.ReverseObject",
    "v2ray.config.FakeDNSObject",
    "v2ray.config.BrowserForwarderObject",
    "v2ray.config.ObservatoryObject",
    "v2ray.config.V2RayConfig",
];
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../../proto/acolors.proto");
    println!("cargo:rerun-if-changed=../../proto/v2ray_config.proto");
    tonic_build::compile_protos("../../proto/acolors.proto")?;

    compile_v2ray_protos()?;

    Ok(())
}

fn compile_v2ray_protos() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = tonic_build::configure()
        .type_attribute(".", "#[derive(serde::Serialize, serde::Deserialize)]")
        .field_attribute(
            "v2ray.config.OutboundObject.OutboundSettings.kind",
            "#[serde(flatten)]",
        )
        .field_attribute(
            "v2ray.config.InboundObject.InboundSettings.kind",
            "#[serde(flatten)]",
        )
        .type_attribute(".", "#[serde(rename_all = \"camelCase\")]");
    for &object in STRUCT_PATHS {
        builder = builder.type_attribute(object, "#[serde(default)]");
    }
    builder.compile(&["../../proto/v2ray_config.proto"], &["../../proto"])?;
    Ok(())
}
