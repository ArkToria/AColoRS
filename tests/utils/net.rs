use serialize_tool::serialize::serializetool::get_nodes_from_base64;

#[tokio::test]
async fn test_get_nodes_from_base64() -> anyhow::Result<()> {
    let nodes = get_nodes_from_base64("c3M6Ly9ZV1Z6TFRJMU5pMW5ZMjA2ZEdWemRETT1AdGVzdDI6MTIzI3Rlc3QxCnRyb2phbjovL3Rlc3QzQHRlc3QyOjc1Nj9zbmk9dGVzdDQmYWxsb3dpbnNlY3VyZT1mYWxzZSZhbHBuPWgyJTBBaHR0cC8xLjEjdGVzdDEKdm1lc3M6Ly9leUpoWkdRaU9pSjBaWE4wTWlJc0ltRnBaQ0k2TXpFeUxDSm9iM04wSWpvaVptUWlMQ0pwWkNJNkluUmxjM1F6SWl3aWJtVjBJam9pZDNNaUxDSndZWFJvSWpvaVlXWmtJaXdpY0c5eWRDSTZNVFF5TENKd2N5STZJblJsYzNReElpd2ljMk41SWpvaVkyaGhZMmhoTWpBdGNHOXNlVEV6TURVaUxDSnpibWtpT2lJME1USWlMQ0owYkhNaU9pSjBiSE1pTENKMGVYQmxJam9pYm05dVpTSXNJbllpT2lJeUluMD1ACg==")?;
    nodes.into_iter().for_each(|node| {
        println!("{}", node.raw);
    });
    Ok(())
}
