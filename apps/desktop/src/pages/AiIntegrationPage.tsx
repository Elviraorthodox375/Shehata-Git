import { useQuery } from "@tanstack/react-query";
import { Bot, CheckCircle2, Copy, XCircle } from "lucide-react";
import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { getMcpInfo } from "@/lib/tauri";

/**
 * AI Integration page.
 * Shows the MCP server status and provides the configuration snippet that
 * AI coding clients (Cursor, Claude Code, Codex, …) use to talk to
 * shehata-mcp over stdio.
 */
export function AiIntegrationPage() {
  const mcp = useQuery({ queryKey: ["mcp-info"], queryFn: getMcpInfo });
  const [copied, setCopied] = useState(false);

  async function copyConfig() {
    if (!mcp.data) return;
    await navigator.clipboard.writeText(mcp.data.config_snippet);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  }

  return (
    <div className="mx-auto max-w-2xl space-y-4">
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2.5">
              <Bot className="h-5 w-5 text-primary" aria-hidden />
              <CardTitle>Shehata MCP server</CardTitle>
            </div>
            {mcp.data && (
              <Badge variant={mcp.data.available ? "success" : "warning"}>
                {mcp.data.available ? (
                  <>
                    <CheckCircle2 className="h-3 w-3" aria-hidden /> available
                  </>
                ) : (
                  <>
                    <XCircle className="h-3 w-3" aria-hidden /> not built yet
                  </>
                )}
              </Badge>
            )}
          </div>
          <CardDescription>
            Lets AI coding assistants check status, commit, pull, and push through the correct
            account — with the same safety rules as this app.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          {mcp.data?.executable_path && (
            <div>
              <p className="mb-1 text-xs font-medium text-muted-foreground">Executable</p>
              <code className="block truncate rounded-md border border-border bg-background px-3 py-2 font-mono text-xs">
                {mcp.data.executable_path}
              </code>
            </div>
          )}

          {mcp.data && (
            <div>
              <div className="mb-1 flex items-center justify-between">
                <p className="text-xs font-medium text-muted-foreground">Client configuration</p>
                <Button variant="ghost" size="sm" onClick={copyConfig}>
                  <Copy aria-hidden />
                  {copied ? "Copied" : "Copy config"}
                </Button>
              </div>
              <pre className="overflow-x-auto rounded-md border border-border bg-background px-3 py-2 font-mono text-xs leading-relaxed">
                {mcp.data.config_snippet}
              </pre>
            </div>
          )}
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle className="text-sm">Safety rules for AI tools</CardTitle>
        </CardHeader>
        <CardContent>
          <ul className="list-inside list-disc space-y-1 text-sm text-muted-foreground">
            <li>AI tools can read status, commit, pull (fast-forward only), and push normally.</li>
            <li>Force push, remote deletion, and destructive resets are never exposed.</li>
            <li>Every push uses the account you assigned to that repository.</li>
            <li>Repositories can require your approval before an AI push.</li>
            <li>Tokens never appear in tool results — credentials stay in the GitHub CLI.</li>
          </ul>
        </CardContent>
      </Card>
    </div>
  );
}
