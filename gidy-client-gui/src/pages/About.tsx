import { Info, Shield, Code, ExternalLink } from "lucide-react";

const APP_VERSION = "v0.2.7";

export default function About() {
  return (
    <div className="space-y-5">
      <div className="bg-card rounded-2xl border border-border p-7">
        <div className="flex items-center gap-4 mb-6">
          <img
            src="/logo.png"
            alt="gidy"
            className="w-16 h-16 rounded-xl ring-1 ring-border"
          />
          <div>
            <h2 className="text-xl font-bold">gidy client</h2>
            <p className="text-sm text-muted-foreground tabular mt-0.5">
              {APP_VERSION}
            </p>
          </div>
        </div>

        <div className="space-y-3.5 text-sm text-muted-foreground">
          <div className="flex items-start gap-3">
            <Shield
              size={15}
              strokeWidth={1.75}
              className="text-foreground mt-0.5 shrink-0"
            />
            <span>
              gidy protocol — QUIC-based secure proxy with protocol morphing
            </span>
          </div>
          <div className="flex items-start gap-3">
            <Code
              size={15}
              strokeWidth={1.75}
              className="text-foreground mt-0.5 shrink-0"
            />
            <span>Built with Tauri + React + Rust</span>
          </div>
          <div className="flex items-start gap-3">
            <Info
              size={15}
              strokeWidth={1.75}
              className="text-foreground mt-0.5 shrink-0"
            />
            <span>Client for secure, obfuscated proxy tunneling</span>
          </div>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-4">
        <div className="bg-card rounded-2xl border border-border p-6">
          <h3 className="text-sm font-semibold mb-2">License</h3>
          <p className="text-xs text-muted-foreground">
            Proprietary. All rights reserved.
          </p>
        </div>
        <a
          href="https://github.com/yokeay/gidy-client"
          target="_blank"
          rel="noreferrer"
          className="bg-card rounded-2xl border border-border p-6 hover:bg-muted/40 transition-colors group"
        >
          <div className="flex items-center justify-between mb-2">
            <h3 className="text-sm font-semibold">Repository</h3>
            <ExternalLink
              size={13}
              strokeWidth={1.75}
              className="text-muted-foreground group-hover:text-foreground transition-colors"
            />
          </div>
          <p className="text-xs text-muted-foreground tabular">
            github.com/yokeay/gidy-client
          </p>
        </a>
      </div>
    </div>
  );
}
