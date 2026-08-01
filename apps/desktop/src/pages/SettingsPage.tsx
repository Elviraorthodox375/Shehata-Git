import { Moon, Sun } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";

interface SettingsPageProps {
  theme: "dark" | "light";
  onThemeChange: (theme: "dark" | "light") => void;
}

export function SettingsPage({ theme, onThemeChange }: SettingsPageProps) {
  return (
    <div className="mx-auto max-w-2xl space-y-4">
      <Card>
        <CardHeader>
          <CardTitle>Appearance</CardTitle>
          <CardDescription>
            Dark is the Shehata Git default. Light theme is fully supported.
          </CardDescription>
        </CardHeader>
        <CardContent className="flex gap-2">
          <Button
            variant={theme === "dark" ? "default" : "outline"}
            size="sm"
            onClick={() => onThemeChange("dark")}
            aria-pressed={theme === "dark"}
          >
            <Moon aria-hidden />
            Dark
          </Button>
          <Button
            variant={theme === "light" ? "default" : "outline"}
            size="sm"
            onClick={() => onThemeChange("light")}
            aria-pressed={theme === "light"}
          >
            <Sun aria-hidden />
            Light
          </Button>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>Push safety</CardTitle>
          <CardDescription>
            Default policy for newly linked repositories. You can change it per repository later.
          </CardDescription>
        </CardHeader>
        <CardContent>
          <p className="text-sm text-muted-foreground">
            Normal pushes are allowed. Force pushes, remote branch deletion, and destructive resets
            are always blocked — this is not configurable, on purpose.
          </p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader>
          <CardTitle>About</CardTitle>
        </CardHeader>
        <CardContent className="space-y-1 text-sm text-muted-foreground">
          <p>Shehata Git v0.1.0 — early development build.</p>
          <p>Local-first. No accounts leave this machine. MIT licensed.</p>
        </CardContent>
      </Card>
    </div>
  );
}
