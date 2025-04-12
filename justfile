set shell := ["nu", "-c"]

check-examples:
    cd examples; ls *.rs | each { cargo check --example $"($in.name | split row '.' | first)" }
    
