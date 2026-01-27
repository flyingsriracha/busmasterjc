use busmaster_db::dbc::DbcParser;

#[test]
fn diagnostic_parse_simple_dbc() {
    let dbc = r#"VERSION ""

NS_ :

BS_:

BU_: ECU1 ECU2

BO_ 100 TestMessage: 8 ECU1
 SG_ TestSignal : 0|8@1+ (1,0) [0|255] "" ECU2

"#;

    println!("=== DIAGNOSTIC TEST ===");
    println!("Input DBC:\n{}", dbc);

    let result = DbcParser::parse(dbc);

    match &result {
        Ok(db) => {
            println!("\n=== PARSE SUCCESS ===");
            println!("Version: '{}'", db.version);
            println!("Nodes: {:?}", db.nodes);
            println!("Number of messages: {}", db.messages.len());

            for (i, msg) in db.messages.iter().enumerate() {
                println!("\nMessage {}:", i);
                println!("  ID: {}", msg.id);
                println!("  Name: {}", msg.name);
                println!("  Length: {}", msg.length);
                println!("  Transmitter: {}", msg.transmitter);
                println!("  Number of signals: {}", msg.signals.len());

                for (j, sig) in msg.signals.iter().enumerate() {
                    println!("\n  Signal {}:", j);
                    println!("    Name: {}", sig.name);
                    println!("    Start bit: {}", sig.start_bit);
                    println!("    Length: {}", sig.length);
                    println!("    Byte order: {:?}", sig.byte_order);
                    println!("    Value type: {:?}", sig.value_type);
                }
            }
        },
        Err(e) => {
            println!("\n=== PARSE FAILED ===");
            println!("Error: {:?}", e);
        },
    }

    assert!(result.is_ok(), "Parse should succeed");
    let db = result.unwrap();

    println!("\n=== ASSERTIONS ===");
    println!("Expected messages: 1, Got: {}", db.messages.len());

    if db.messages.is_empty() {
        panic!("NO MESSAGES PARSED! This is the bug.");
    }

    assert_eq!(db.messages.len(), 1, "Should have 1 message");
    assert_eq!(db.messages[0].signals.len(), 1, "Should have 1 signal");
}
