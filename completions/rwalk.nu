module completions {

  def "nu-complete rwalk mode" [] {
    [ "recursive" "recursion" "r" "classic" "c" ]
  }

  def "nu-complete rwalk show" [] {
    [ "length" "size" "hash" "md5" "headers_length" "headers_hash" "body" "content" "text" "headers" "cookie" "cookies" "type" ]
  }

  # A blazing fast web directory scanner
  export extern rwalk [
    url?: string              # Target URL
    ...wordlists: string      # Wordlist(s)
    --mode(-m): string@"nu-complete rwalk mode" # Crawl mode
    --force                   # Force scan even if the target is not responding
    --hit-connection-errors   # Consider connection errors as a hit
    --hce                     # Consider connection errors as a hit
    --threads(-t): string     # Number of threads to use
    --depth(-d): string       # Crawl recursively until given depth
    --output(-o): string      # Output file
    --timeout: string         # Request timeout in seconds
    --to: string              # Request timeout in seconds
    --user-agent(-u): string  # User agent
    --method(-X): string      # HTTP method
    --data(-D): string        # Data to send with the request
    --headers(-H): string     # Headers to send
    --cookies(-C): string     # Cookies to send
    --follow-redirects(-R): string # Follow redirects
    --config(-c): string      # Configuration file
    --throttle: string        # Request throttling (requests per second) per thread
    --max-time(-M): string    # Max time to run (will abort after given time) in seconds
    --no-color                # Don't use colors You can also set the NO_COLOR environment variable
    --quiet(-q)               # Quiet mode
    --interactive(-i)         # Interactive mode
    --insecure                # Insecure mode, disables SSL certificate validation
    --unsecure                # Insecure mode, disables SSL certificate validation
    --show: string@"nu-complete rwalk show" # Show response additional body information
    --resume(-r)              # Resume from a saved file
    --save-file: string       # Custom save file
    --no-save                 # Don't save the state in case you abort
    --keep-save               # Keep the save file after finishing when using --resume
    --keep                    # Keep the save file after finishing when using --resume
    --transform(-T): string   # Wordlist transformations: "lower", "upper", "prefix", "suffix", "capitalize", "reverse", "remove", "replace"
    --wordlist-filter(-w): string # Wordlist filtering: "contains", "starts", "ends", "regex", "length"
    --wf: string              # Wordlist filtering: "contains", "starts", "ends", "regex", "length"
    --filter(-f): string      # Response filtering: "time", "status", "contains", "starts", "end", "regex", "length", "hash", "header", "json", "depth", "type"
    --or                      # Treat filters as or instead of and
    --force-recursion         # Force the recursion over non-directories
    --fr                      # Force the recursion over non-directories
    --request-file: string    # Request file (.http, .rest)
    --rf: string              # Request file (.http, .rest)
    --proxy(-P): string       # Proxy URL
    --proxy-auth: string      # Proxy username and password
    --generate-markdown       # Generate markdown help - for developers
    --generate-completions    # Generate shell completions - for developers
    --help(-h)                # Print help
    --version(-V)             # Print version
  ]

}

export use completions *
