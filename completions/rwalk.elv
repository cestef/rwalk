
use builtin;
use str;

set edit:completion:arg-completer[rwalk] = {|@words|
    fn spaces {|n|
        builtin:repeat $n ' ' | str:join ''
    }
    fn cand {|text desc|
        edit:complex-candidate $text &display=$text' '(spaces (- 14 (wcswidth $text)))$desc
    }
    var command = 'rwalk'
    for word $words[1..-1] {
        if (str:has-prefix $word '-') {
            break
        }
        set command = $command';'$word
    }
    var completions = [
        &'rwalk'= {
            cand -m 'Crawl mode'
            cand --mode 'Crawl mode'
            cand -t 'Number of threads to use'
            cand --threads 'Number of threads to use'
            cand -d 'Crawl recursively until given depth'
            cand --depth 'Crawl recursively until given depth'
            cand -o 'Output file'
            cand --output 'Output file'
            cand --timeout 'Request timeout in seconds'
            cand --to 'Request timeout in seconds'
            cand -u 'User agent'
            cand --user-agent 'User agent'
            cand -X 'HTTP method'
            cand --method 'HTTP method'
            cand -D 'Data to send with the request'
            cand --data 'Data to send with the request'
            cand -H 'Headers to send'
            cand --headers 'Headers to send'
            cand -C 'Cookies to send'
            cand --cookies 'Cookies to send'
            cand -R 'Follow redirects'
            cand --follow-redirects 'Follow redirects'
            cand -c 'Configuration file'
            cand --config 'Configuration file'
            cand --throttle 'Request throttling (requests per second) per thread'
            cand -M 'Max time to run (will abort after given time) in seconds'
            cand --max-time 'Max time to run (will abort after given time) in seconds'
            cand --show 'Show response additional body information'
            cand --save-file 'Custom save file'
            cand -T 'Wordlist transformations: "lower", "upper", "prefix", "suffix", "capitalize", "reverse", "remove", "replace"'
            cand --transform 'Wordlist transformations: "lower", "upper", "prefix", "suffix", "capitalize", "reverse", "remove", "replace"'
            cand -w 'Wordlist filtering: "contains", "starts", "ends", "regex", "length"'
            cand --wordlist-filter 'Wordlist filtering: "contains", "starts", "ends", "regex", "length"'
            cand --wf 'Wordlist filtering: "contains", "starts", "ends", "regex", "length"'
            cand -f 'Response filtering: "time", "status", "contains", "starts", "end", "regex", "length", "hash", "header", "json", "depth", "type"'
            cand --filter 'Response filtering: "time", "status", "contains", "starts", "end", "regex", "length", "hash", "header", "json", "depth", "type"'
            cand --request-file 'Request file (.http, .rest)'
            cand --rf 'Request file (.http, .rest)'
            cand -P 'Proxy URL'
            cand --proxy 'Proxy URL'
            cand --proxy-auth 'Proxy username and password'
            cand --force 'Force scan even if the target is not responding'
            cand --hit-connection-errors 'Consider connection errors as a hit'
            cand --hce 'Consider connection errors as a hit'
            cand --no-color 'Don''t use colors You can also set the NO_COLOR environment variable'
            cand -q 'Quiet mode'
            cand --quiet 'Quiet mode'
            cand -i 'Interactive mode'
            cand --interactive 'Interactive mode'
            cand --insecure 'Insecure mode, disables SSL certificate validation'
            cand --unsecure 'Insecure mode, disables SSL certificate validation'
            cand -r 'Resume from a saved file'
            cand --resume 'Resume from a saved file'
            cand --no-save 'Don''t save the state in case you abort'
            cand --keep-save 'Keep the save file after finishing when using --resume'
            cand --keep 'Keep the save file after finishing when using --resume'
            cand --or 'Treat filters as or instead of and'
            cand --force-recursion 'Force the recursion over non-directories'
            cand --fr 'Force the recursion over non-directories'
            cand --generate-markdown 'Generate markdown help - for developers'
            cand --generate-completions 'Generate shell completions - for developers'
            cand -h 'Print help'
            cand --help 'Print help'
            cand -V 'Print version'
            cand --version 'Print version'
        }
    ]
    $completions[$command]
}
