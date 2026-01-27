/// QA Investigation - DBC Parser Signal Parsing
///
/// Testing from a QA perspective with various edge cases and scenarios
use busmaster_db::dbc::DbcParser;

#[test]
fn qa_test_1_minimal_signal() {
    println!("\n=== QA TEST 1: Minimal Signal ===");
    let dbc = r#"VERSION ""
BU_: ECU1
BO_ 100 Msg: 8 ECU1
 SG_ Sig : 0|8@1+ (1,0) [0|255] "" ECU1
"#;

    println!("Input:\n{}", dbc);
    let result = DbcParser::parse(dbc);

    match &result {
        Ok(db) => {
            println!("✓ Parse succeeded");
            println!("  Messages: {}", db.messages.len());
            if !db.messages.is_empty() {
                println!("  Signals in msg[0]: {}", db.messages[0].signals.len());
            }
        },
        Err(e) => println!("✗ Parse failed: {:?}", e),
    }

    assert!(result.is_ok());
}

#[test]
fn qa_test_2_no_leading_space() {
    println!("\n=== QA TEST 2: Signal with NO leading space ===");
    let dbc = r#"VERSION ""
BU_: ECU1
BO_ 100 Msg: 8 ECU1
SG_ Sig : 0|8@1+ (1,0) [0|255] "" ECU1
"#;

    println!("Input (no space before SG_):\n{}", dbc);
    let result = DbcParser::parse(dbc);

    match &result {
        Ok(db) => {
            println!("✓ Parse succeeded");
            println!("  Messages: {}", db.messages.len());
            if !db.messages.is_empty() {
                println!("  Signals in msg[0]: {}", db.messages[0].signals.len());
            }
        },
        Err(e) => println!("✗ Parse failed: {:?}", e),
    }
}

#[test]
fn qa_test_3_multiple_spaces() {
    println!("\n=== QA TEST 3: Signal with MULTIPLE leading spaces ===");
    let dbc = r#"VERSION ""
BU_: ECU1
BO_ 100 Msg: 8 ECU1
   SG_ Sig : 0|8@1+ (1,0) [0|255] "" ECU1
"#;

    println!("Input (3 spaces before SG_):\n{}", dbc);
    let result = DbcParser::parse(dbc);

    match &result {
        Ok(db) => {
            println!("✓ Parse succeeded");
            println!("  Messages: {}", db.messages.len());
            if !db.messages.is_empty() {
                println!("  Signals in msg[0]: {}", db.messages[0].signals.len());
            }
        },
        Err(e) => println!("✗ Parse failed: {:?}", e),
    }
}

#[test]
fn qa_test_4_tab_character() {
    println!("\n=== QA TEST 4: Signal with TAB character ===");
    let dbc = "VERSION \"\"\nBU_: ECU1\nBO_ 100 Msg: 8 ECU1\n\tSG_ Sig : 0|8@1+ (1,0) [0|255] \"\" ECU1\n";

    println!("Input (tab before SG_)");
    let result = DbcParser::parse(dbc);

    match &result {
        Ok(db) => {
            println!("✓ Parse succeeded");
            println!("  Messages: {}", db.messages.len());
            if !db.messages.is_empty() {
                println!("  Signals in msg[0]: {}", db.messages[0].signals.len());
            }
        },
        Err(e) => println!("✗ Parse failed: {:?}", e),
    }
}

#[test]
fn qa_test_5_multiple_signals() {
    println!("\n=== QA TEST 5: Multiple signals ===");
    let dbc = r#"VERSION ""
BU_: ECU1
BO_ 100 Msg: 8 ECU1
 SG_ Sig1 : 0|8@1+ (1,0) [0|255] "" ECU1
 SG_ Sig2 : 8|8@1+ (1,0) [0|255] "" ECU1
 SG_ Sig3 : 16|16@1+ (1,0) [0|65535] "" ECU1
"#;

    println!("Input (3 signals):\n{}", dbc);
    let result = DbcParser::parse(dbc);

    match &result {
        Ok(db) => {
            println!("✓ Parse succeeded");
            println!("  Messages: {}", db.messages.len());
            if !db.messages.is_empty() {
                println!("  Signals in msg[0]: {}", db.messages[0].signals.len());
                for (i, sig) in db.messages[0].signals.iter().enumerate() {
                    println!("    Signal {}: {}", i, sig.name);
                }
            }
        },
        Err(e) => println!("✗ Parse failed: {:?}", e),
    }
}

#[test]
fn qa_test_6_message_without_signals() {
    println!("\n=== QA TEST 6: Message with NO signals ===");
    let dbc = r#"VERSION ""
BU_: ECU1
BO_ 100 Msg: 8 ECU1
BO_ 200 Msg2: 8 ECU1
 SG_ Sig : 0|8@1+ (1,0) [0|255] "" ECU1
"#;

    println!("Input (first message has no signals):\n{}", dbc);
    let result = DbcParser::parse(dbc);

    match &result {
        Ok(db) => {
            println!("✓ Parse succeeded");
            println!("  Messages: {}", db.messages.len());
            for (i, msg) in db.messages.iter().enumerate() {
                println!(
                    "  Message {}: {} - {} signals",
                    i,
                    msg.name,
                    msg.signals.len()
                );
            }
        },
        Err(e) => println!("✗ Parse failed: {:?}", e),
    }
}

#[test]
fn qa_test_7_real_world_format() {
    println!("\n=== QA TEST 7: Real-world DBC format ===");
    // This is closer to what Vector CANdb++ actually generates
    let dbc = r#"VERSION ""


NS_ : 
	NS_DESC_
	CM_
	BA_DEF_
	BA_
	VAL_
	CAT_DEF_
	CAT_
	FILTER
	BA_DEF_DEF_
	EV_DATA_
	ENVVAR_DATA_
	SGTYPE_
	SGTYPE_VAL_
	BA_DEF_SGTYPE_
	BA_SGTYPE_
	SIG_TYPE_REF_
	VAL_TABLE_
	SIG_GROUP_
	SIG_VALTYPE_
	SIGTYPE_VALTYPE_
	BO_TX_BU_
	BA_DEF_REL_
	BA_REL_
	BA_SGTYPE_REL_
	SG_MUL_VAL_

BS_:

BU_: ECU1 ECU2

BO_ 100 EngineData: 8 ECU1
 SG_ EngineSpeed : 0|16@1+ (0.25,0) [0|16383.75] "rpm" ECU2
 SG_ EngineTemp : 16|8@1+ (1,-40) [-40|215] "degC" ECU2

"#;

    println!("Input (real-world format with NS_ section)");
    let result = DbcParser::parse(dbc);

    match &result {
        Ok(db) => {
            println!("✓ Parse succeeded");
            println!("  Messages: {}", db.messages.len());
            if !db.messages.is_empty() {
                println!("  Signals in msg[0]: {}", db.messages[0].signals.len());
                for sig in &db.messages[0].signals {
                    println!(
                        "    - {} ({}|{}@{}{})",
                        sig.name,
                        sig.start_bit,
                        sig.length,
                        if sig.byte_order == busmaster_core::ByteOrder::LittleEndian {
                            "1"
                        } else {
                            "0"
                        },
                        if sig.value_type == busmaster_core::ValueType::Unsigned {
                            "+"
                        } else {
                            "-"
                        }
                    );
                }
            }
        },
        Err(e) => println!("✗ Parse failed: {:?}", e),
    }
}

#[test]
fn qa_test_8_windows_line_endings() {
    println!("\n=== QA TEST 8: Windows line endings (CRLF) ===");
    let dbc = "VERSION \"\"\r\nBU_: ECU1\r\nBO_ 100 Msg: 8 ECU1\r\n SG_ Sig : 0|8@1+ (1,0) [0|255] \"\" ECU1\r\n";

    println!("Input (CRLF line endings)");
    let result = DbcParser::parse(dbc);

    match &result {
        Ok(db) => {
            println!("✓ Parse succeeded");
            println!("  Messages: {}", db.messages.len());
            if !db.messages.is_empty() {
                println!("  Signals in msg[0]: {}", db.messages[0].signals.len());
            }
        },
        Err(e) => println!("✗ Parse failed: {:?}", e),
    }
}

#[test]
fn qa_test_9_mixed_whitespace() {
    println!("\n=== QA TEST 9: Mixed whitespace (spaces and tabs) ===");
    let dbc = "VERSION \"\"\nBU_: ECU1\nBO_ 100 Msg: 8 ECU1\n \tSG_ Sig : 0|8@1+ (1,0) [0|255] \"\" ECU1\n";

    println!("Input (space + tab before SG_)");
    let result = DbcParser::parse(dbc);

    match &result {
        Ok(db) => {
            println!("✓ Parse succeeded");
            println!("  Messages: {}", db.messages.len());
            if !db.messages.is_empty() {
                println!("  Signals in msg[0]: {}", db.messages[0].signals.len());
            }
        },
        Err(e) => println!("✗ Parse failed: {:?}", e),
    }
}

#[test]
fn qa_test_10_blank_lines_between() {
    println!("\n=== QA TEST 10: Blank lines between signals ===");
    let dbc = r#"VERSION ""
BU_: ECU1
BO_ 100 Msg: 8 ECU1
 SG_ Sig1 : 0|8@1+ (1,0) [0|255] "" ECU1

 SG_ Sig2 : 8|8@1+ (1,0) [0|255] "" ECU1
"#;

    println!("Input (blank line between signals):\n{}", dbc);
    let result = DbcParser::parse(dbc);

    match &result {
        Ok(db) => {
            println!("✓ Parse succeeded");
            println!("  Messages: {}", db.messages.len());
            if !db.messages.is_empty() {
                println!("  Signals in msg[0]: {}", db.messages[0].signals.len());
            }
        },
        Err(e) => println!("✗ Parse failed: {:?}", e),
    }
}
