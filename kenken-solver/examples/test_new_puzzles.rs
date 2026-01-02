use kenken_core::format::sgt_desc::parse_keen_desc;
use kenken_core::rules::Ruleset;

fn main() {
    let rules = Ruleset::keen_baseline();

    let puzzles = vec![
        (2, "b__,a3a3", "2x2_add"),
        (3, "abc,def,ghi", "3x3_simple"),
        (4, "a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p", "4x4_trivial"),
        (5, "a,b,c,d,e,f,g,h,i,j,k,l,m,n,o,p,q,r,s,t,u,v,w,x,y", "5x5_trivial"),
    ];

    for (n, desc, label) in puzzles {
        eprintln!("\nTesting: {} ({})", label, desc);
        match parse_keen_desc(n, desc) {
            Ok(puzzle) => {
                eprintln!("  Parsed OK");
                match puzzle.validate(rules) {
                    Ok(_) => {
                        eprintln!("  Validated OK");
                    }
                    Err(e) => eprintln!("  Validation error: {:?}", e),
                }
            }
            Err(e) => eprintln!("  Parse error: {:?}", e),
        }
    }
}
