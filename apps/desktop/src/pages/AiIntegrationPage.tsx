import { useMutation, useQuery } from "@tanstack/react-query";
import { Bot, CheckCircle2, Copy, FileCode2, Loader2, XCircle } from "lucide-react";
import { useState } from "react";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { generateRepositoryAgents, getMcpInfo, listRepositories } from "@/lib/tauri";

/**
 * AI Integration page.
 * Shows the MCP server status and provides the configuration snippet that
 * AI coding clients (Cursor, Claude Code, Codex, …) use to talk to
 * shehata-mcp over stdio.
 */
export function AiIntegrationPage() {
  const mcp = useQuery({ queryKey: ["mcp-info"], queryFn: getMcpInfo });
  const repositories = useQuery({ queryKey: ["repositories"], queryFn: listRepositories });
  const [copied, setCopied] = useState(false);
  const [repositoryId, setRepositoryId] = useState("");
  const generateAgents = useMutation({ mutationFn: generateRepositoryAgents });
  const selectedId = repositoryId || repositories.data?.[0]?.id || "";

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
          <div className="flex items-center gap-2.5">
            <FileCode2 className="h-5 w-5 text-primary" aria-hidden />
            <CardTitle>Repository instructions</CardTitle>
          </div>
          <CardDescription>
            Add or safely update the bounded Shehata Git section in AGENTS.md without replacing
            existing project instructions.
          </CardDescription>
        </CardHeader>
        <CardContent className="space-y-3">
          <div className="flex flex-col gap-2 sm:flex-row">
            <select
              value={selectedId}
              onChange={(event) => {
                setRepositoryId(event.target.value);
                generateAgents.reset();
              }}
              disabled={!repositories.data?.length || generateAgents.isPending}
              className="h-10 min-w-0 flex-1 rounded-md border border-input bg-background px-3 text-sm outline-none focus:border-primary"
            >
              {repositories.data?.length ? (
                repositories.data.map((repository) => (
                  <option key={repository.id} value={repository.id}>
                    {repository.display_name} — {repository.canonical_path}
                  </option>
                ))
              ) : (
                <option value="">No registered repositories</option>
              )}
            </select>
            <Button
              onClick={() => generateAgents.mutate(selectedId)}
              disabled={!selectedId || generateAgents.isPending}
            >
              {generateAgents.isPending ? (
                <Loader2 className="animate-spin" aria-hidden />
              ) : (
                <FileCode2 aria-hidden />
              )}
              Generate AGENTS.md
            </Button>
          </div>
          {generateAgents.data && (
            <p className="text-sm text-success">
              {generateAgents.data.created ? "Created" : "Updated"}: {generateAgents.data.path}
            </p>
          )}
          {generateAgents.isError && (
            <p className="text-sm text-destructive">
              {generateAgents.error instanceof Error
                ? generateAgents.error.message
                : String(generateAgents.error)}
            </p>
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
