use qlty_coverage::parser::Clover;
use qlty_coverage::Parser;

#[test]
fn clover_results() {
    // Make sure that the <?xml version="1.0"?> tag is always right at the beginning of the string to avoid parsing errors
    let input = include_str!("fixtures/clover/sample.xml");

    let parsed_results = Clover::new().parse_text(input).unwrap();
    insta::assert_yaml_snapshot!(parsed_results, @r#"
    - path: /app/Console/Kernel.php
      hits:
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "0"
        - "-1"
        - "-1"
        - "0"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "1"
        - "-1"
        - "1"
        - "-1"
        - "1"
        - "-1"
        - "-1"
        - "-1"
    - path: /app/Exceptions/Handler.php
      hits:
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "1"
        - "-1"
        - "1"
        - "-1"
        - "1"
        - "-1"
        - "-1"
        - "-1"
    - path: /app/Http/Controllers/Controller.php
      hits:
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
    - path: /app/Http/Kernel.php
      hits:
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
    - path: /app/Http/Middleware/Authenticate.php
      hits:
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "0"
        - "-1"
        - "0"
        - "-1"
        - "-1"
        - "-1"
    - path: /app/Http/Middleware/EncryptCookies.php
      hits:
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
    - path: /app/Http/Middleware/PreventRequestsDuringMaintenance.php
      hits:
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
    - path: /app/Http/Middleware/RedirectIfAuthenticated.php
      hits:
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "0"
        - "-1"
        - "0"
        - "-1"
        - "0"
        - "0"
        - "0"
        - "-1"
        - "-1"
        - "-1"
        - "0"
        - "-1"
        - "-1"
        - "-1"
    - path: /app/Http/Middleware/TrimStrings.php
      hits:
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
    - path: /app/Http/Middleware/TrustHosts.php
      hits:
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "0"
        - "-1"
        - "0"
        - "0"
        - "0"
        - "-1"
        - "-1"
        - "-1"
    - path: /app/Http/Middleware/TrustProxies.php
      hits:
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
    - path: /app/Http/Middleware/ValidateSignature.php
      hits:
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
    - path: /app/Http/Middleware/VerifyCsrfToken.php
      hits:
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
    - path: /app/Models/User.php
      hits:
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
    - path: /app/Providers/AppServiceProvider.php
      hits:
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "1"
        - "-1"
        - "-1"
        - "1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "1"
        - "-1"
        - "-1"
        - "1"
        - "-1"
        - "-1"
    - path: /app/Providers/AuthServiceProvider.php
      hits:
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "1"
        - "-1"
        - "-1"
        - "1"
        - "-1"
        - "-1"
    - path: /app/Providers/BroadcastServiceProvider.php
      hits:
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "0"
        - "-1"
        - "0"
        - "-1"
        - "0"
        - "-1"
        - "-1"
        - "-1"
    - path: /app/Providers/EventServiceProvider.php
      hits:
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "1"
        - "-1"
        - "-1"
        - "1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "1"
        - "-1"
        - "1"
        - "-1"
        - "-1"
        - "-1"
    - path: /app/Providers/RouteServiceProvider.php
      hits:
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "-1"
        - "1"
        - "-1"
        - "1"
        - "0"
        - "1"
        - "-1"
        - "1"
        - "1"
        - "1"
        - "1"
        - "-1"
        - "1"
        - "1"
        - "1"
        - "-1"
        - "-1"
        - "-1"
    "#);
}
