
using namespace System.Management.Automation
using namespace System.Management.Automation.Language

Register-ArgumentCompleter -Native -CommandName 'rwalk' -ScriptBlock {
    param($wordToComplete, $commandAst, $cursorPosition)

    $commandElements = $commandAst.CommandElements
    $command = @(
        'rwalk'
        for ($i = 1; $i -lt $commandElements.Count; $i++) {
            $element = $commandElements[$i]
            if ($element -isnot [StringConstantExpressionAst] -or
                $element.StringConstantType -ne [StringConstantType]::BareWord -or
                $element.Value.StartsWith('-') -or
                $element.Value -eq $wordToComplete) {
                break
        }
        $element.Value
    }) -join ';'

    $completions = @(switch ($command) {
        'rwalk' {
            [CompletionResult]::new('-m', 'm', [CompletionResultType]::ParameterName, 'Crawl mode')
            [CompletionResult]::new('--mode', 'mode', [CompletionResultType]::ParameterName, 'Crawl mode')
            [CompletionResult]::new('-t', 't', [CompletionResultType]::ParameterName, 'Number of threads to use')
            [CompletionResult]::new('--threads', 'threads', [CompletionResultType]::ParameterName, 'Number of threads to use')
            [CompletionResult]::new('-d', 'd', [CompletionResultType]::ParameterName, 'Crawl recursively until given depth')
            [CompletionResult]::new('--depth', 'depth', [CompletionResultType]::ParameterName, 'Crawl recursively until given depth')
            [CompletionResult]::new('-o', 'o', [CompletionResultType]::ParameterName, 'Output file')
            [CompletionResult]::new('--output', 'output', [CompletionResultType]::ParameterName, 'Output file')
            [CompletionResult]::new('--timeout', 'timeout', [CompletionResultType]::ParameterName, 'Request timeout in seconds')
            [CompletionResult]::new('--to', 'to', [CompletionResultType]::ParameterName, 'Request timeout in seconds')
            [CompletionResult]::new('-u', 'u', [CompletionResultType]::ParameterName, 'User agent')
            [CompletionResult]::new('--user-agent', 'user-agent', [CompletionResultType]::ParameterName, 'User agent')
            [CompletionResult]::new('-X', 'X ', [CompletionResultType]::ParameterName, 'HTTP method')
            [CompletionResult]::new('--method', 'method', [CompletionResultType]::ParameterName, 'HTTP method')
            [CompletionResult]::new('-D', 'D ', [CompletionResultType]::ParameterName, 'Data to send with the request')
            [CompletionResult]::new('--data', 'data', [CompletionResultType]::ParameterName, 'Data to send with the request')
            [CompletionResult]::new('-H', 'H ', [CompletionResultType]::ParameterName, 'Headers to send')
            [CompletionResult]::new('--headers', 'headers', [CompletionResultType]::ParameterName, 'Headers to send')
            [CompletionResult]::new('-C', 'C ', [CompletionResultType]::ParameterName, 'Cookies to send')
            [CompletionResult]::new('--cookies', 'cookies', [CompletionResultType]::ParameterName, 'Cookies to send')
            [CompletionResult]::new('-R', 'R ', [CompletionResultType]::ParameterName, 'Follow redirects')
            [CompletionResult]::new('--follow-redirects', 'follow-redirects', [CompletionResultType]::ParameterName, 'Follow redirects')
            [CompletionResult]::new('-c', 'c', [CompletionResultType]::ParameterName, 'Configuration file')
            [CompletionResult]::new('--config', 'config', [CompletionResultType]::ParameterName, 'Configuration file')
            [CompletionResult]::new('--throttle', 'throttle', [CompletionResultType]::ParameterName, 'Request throttling (requests per second) per thread')
            [CompletionResult]::new('-M', 'M ', [CompletionResultType]::ParameterName, 'Max time to run (will abort after given time) in seconds')
            [CompletionResult]::new('--max-time', 'max-time', [CompletionResultType]::ParameterName, 'Max time to run (will abort after given time) in seconds')
            [CompletionResult]::new('--show', 'show', [CompletionResultType]::ParameterName, 'Show response additional body information')
            [CompletionResult]::new('--save-file', 'save-file', [CompletionResultType]::ParameterName, 'Custom save file')
            [CompletionResult]::new('-T', 'T ', [CompletionResultType]::ParameterName, 'Wordlist transformations: "lower", "upper", "prefix", "suffix", "capitalize", "reverse", "remove", "replace"')
            [CompletionResult]::new('--transform', 'transform', [CompletionResultType]::ParameterName, 'Wordlist transformations: "lower", "upper", "prefix", "suffix", "capitalize", "reverse", "remove", "replace"')
            [CompletionResult]::new('-w', 'w', [CompletionResultType]::ParameterName, 'Wordlist filtering: "contains", "starts", "ends", "regex", "length"')
            [CompletionResult]::new('--wordlist-filter', 'wordlist-filter', [CompletionResultType]::ParameterName, 'Wordlist filtering: "contains", "starts", "ends", "regex", "length"')
            [CompletionResult]::new('--wf', 'wf', [CompletionResultType]::ParameterName, 'Wordlist filtering: "contains", "starts", "ends", "regex", "length"')
            [CompletionResult]::new('-f', 'f', [CompletionResultType]::ParameterName, 'Response filtering: "time", "status", "contains", "starts", "end", "regex", "length", "hash", "header", "json", "depth", "type"')
            [CompletionResult]::new('--filter', 'filter', [CompletionResultType]::ParameterName, 'Response filtering: "time", "status", "contains", "starts", "end", "regex", "length", "hash", "header", "json", "depth", "type"')
            [CompletionResult]::new('--request-file', 'request-file', [CompletionResultType]::ParameterName, 'Request file (.http, .rest)')
            [CompletionResult]::new('--rf', 'rf', [CompletionResultType]::ParameterName, 'Request file (.http, .rest)')
            [CompletionResult]::new('-P', 'P ', [CompletionResultType]::ParameterName, 'Proxy URL')
            [CompletionResult]::new('--proxy', 'proxy', [CompletionResultType]::ParameterName, 'Proxy URL')
            [CompletionResult]::new('--proxy-auth', 'proxy-auth', [CompletionResultType]::ParameterName, 'Proxy username and password')
            [CompletionResult]::new('--force', 'force', [CompletionResultType]::ParameterName, 'Force scan even if the target is not responding')
            [CompletionResult]::new('--hit-connection-errors', 'hit-connection-errors', [CompletionResultType]::ParameterName, 'Consider connection errors as a hit')
            [CompletionResult]::new('--hce', 'hce', [CompletionResultType]::ParameterName, 'Consider connection errors as a hit')
            [CompletionResult]::new('--no-color', 'no-color', [CompletionResultType]::ParameterName, 'Don''t use colors You can also set the NO_COLOR environment variable')
            [CompletionResult]::new('-q', 'q', [CompletionResultType]::ParameterName, 'Quiet mode')
            [CompletionResult]::new('--quiet', 'quiet', [CompletionResultType]::ParameterName, 'Quiet mode')
            [CompletionResult]::new('-i', 'i', [CompletionResultType]::ParameterName, 'Interactive mode')
            [CompletionResult]::new('--interactive', 'interactive', [CompletionResultType]::ParameterName, 'Interactive mode')
            [CompletionResult]::new('--insecure', 'insecure', [CompletionResultType]::ParameterName, 'Insecure mode, disables SSL certificate validation')
            [CompletionResult]::new('--unsecure', 'unsecure', [CompletionResultType]::ParameterName, 'Insecure mode, disables SSL certificate validation')
            [CompletionResult]::new('-r', 'r', [CompletionResultType]::ParameterName, 'Resume from a saved file')
            [CompletionResult]::new('--resume', 'resume', [CompletionResultType]::ParameterName, 'Resume from a saved file')
            [CompletionResult]::new('--no-save', 'no-save', [CompletionResultType]::ParameterName, 'Don''t save the state in case you abort')
            [CompletionResult]::new('--keep-save', 'keep-save', [CompletionResultType]::ParameterName, 'Keep the save file after finishing when using --resume')
            [CompletionResult]::new('--keep', 'keep', [CompletionResultType]::ParameterName, 'Keep the save file after finishing when using --resume')
            [CompletionResult]::new('--or', 'or', [CompletionResultType]::ParameterName, 'Treat filters as or instead of and')
            [CompletionResult]::new('--force-recursion', 'force-recursion', [CompletionResultType]::ParameterName, 'Force the recursion over non-directories')
            [CompletionResult]::new('--fr', 'fr', [CompletionResultType]::ParameterName, 'Force the recursion over non-directories')
            [CompletionResult]::new('--subdomains', 'subdomains', [CompletionResultType]::ParameterName, 'Allow subdomains to be scanned in spider mode')
            [CompletionResult]::new('--sub', 'sub', [CompletionResultType]::ParameterName, 'Allow subdomains to be scanned in spider mode')
            [CompletionResult]::new('--generate-markdown', 'generate-markdown', [CompletionResultType]::ParameterName, 'Generate markdown help - for developers')
            [CompletionResult]::new('--generate-completions', 'generate-completions', [CompletionResultType]::ParameterName, 'Generate shell completions - for developers')
            [CompletionResult]::new('-h', 'h', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('--help', 'help', [CompletionResultType]::ParameterName, 'Print help')
            [CompletionResult]::new('-V', 'V ', [CompletionResultType]::ParameterName, 'Print version')
            [CompletionResult]::new('--version', 'version', [CompletionResultType]::ParameterName, 'Print version')
            break
        }
    })

    $completions.Where{ $_.CompletionText -like "$wordToComplete*" } |
        Sort-Object -Property ListItemText
}
