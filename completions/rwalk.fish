complete -c rwalk -s m -l mode -d 'Crawl mode' -r -f -a "{recursive	'',recursion	'',r	'',classic	'',c	'',spider	'',s	''}"
complete -c rwalk -s t -l threads -d 'Number of threads to use' -r
complete -c rwalk -s d -l depth -d 'Crawl recursively until given depth' -r
complete -c rwalk -s o -l output -d 'Output file' -r
complete -c rwalk -l timeout -l to -d 'Request timeout in seconds' -r
complete -c rwalk -s u -l user-agent -d 'User agent' -r
complete -c rwalk -s X -l method -d 'HTTP method' -r
complete -c rwalk -s D -l data -d 'Data to send with the request' -r
complete -c rwalk -s H -l headers -d 'Headers to send' -r
complete -c rwalk -s C -l cookies -d 'Cookies to send' -r
complete -c rwalk -s R -l follow-redirects -d 'Follow redirects' -r
complete -c rwalk -s c -l config -d 'Configuration file' -r
complete -c rwalk -l throttle -d 'Request throttling (requests per second) per thread' -r
complete -c rwalk -s M -l max-time -d 'Max time to run (will abort after given time) in seconds' -r
complete -c rwalk -l show -d 'Show response additional body information' -r
complete -c rwalk -l save-file -d 'Custom save file' -r
complete -c rwalk -s T -l transform -d 'Wordlist transformations: "lower", "upper", "prefix", "suffix", "capitalize", "reverse", "remove", "replace"' -r
complete -c rwalk -s w -l wordlist-filter -l wf -d 'Wordlist filtering: "contains", "starts", "ends", "regex", "length"' -r
complete -c rwalk -s f -l filter -d 'Response filtering: "time", "status", "contains", "starts", "end", "regex", "length", "hash", "header", "json", "depth", "type"' -r
complete -c rwalk -l request-file -l rf -d 'Request file (.http, .rest)' -r
complete -c rwalk -s P -l proxy -d 'Proxy URL' -r
complete -c rwalk -l proxy-auth -d 'Proxy username and password' -r
complete -c rwalk -l force -d 'Force scan even if the target is not responding'
complete -c rwalk -l hit-connection-errors -l hce -d 'Consider connection errors as a hit'
complete -c rwalk -l no-color -d 'Don\'t use colors You can also set the NO_COLOR environment variable'
complete -c rwalk -s q -l quiet -d 'Quiet mode'
complete -c rwalk -s i -l interactive -d 'Interactive mode'
complete -c rwalk -l insecure -l unsecure -d 'Insecure mode, disables SSL certificate validation'
complete -c rwalk -s r -l resume -d 'Resume from a saved file'
complete -c rwalk -l no-save -d 'Don\'t save the state in case you abort'
complete -c rwalk -l keep-save -l keep -d 'Keep the save file after finishing when using --resume'
complete -c rwalk -l or -d 'Treat filters as or instead of and'
complete -c rwalk -l force-recursion -l fr -d 'Force the recursion over non-directories'
complete -c rwalk -l subdomains -l sub -d 'Allow subdomains to be scanned in spider mode'
complete -c rwalk -l generate-markdown -d 'Generate markdown help - for developers'
complete -c rwalk -l generate-completions -d 'Generate shell completions - for developers'
complete -c rwalk -s h -l help -d 'Print help'
complete -c rwalk -s V -l version -d 'Print version'
