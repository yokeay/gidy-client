import { Info, Shield, Code } from "lucide-react";

export default function About() {

  return (
    <div className="space-y-6">
      <div className="bg-card rounded-xl border border-border p-6">
        <div className="flex items-center gap-4 mb-6">
          <img src="/logo.png" alt="gidy" className="w-16 h-16 rounded-xl" />
          <div>
            <h2 className="text-xl font-bold">gidy client</h2>
            <p className="text-sm text-muted-foreground">v0.2.3</p>
          </div>
        </div>

        <div className="space-y-4 text-sm text-muted-foreground">
          <div className="flex items-center gap-3">
            <Shield size={16} className="text-primary" />
            <span>gidy protocol — QUIC-based secure proxy with protocol morphing</span>
          </div>
          <div className="flex items-center gap-3">
            <Code size={16} className="text-primary" />
            <span>Built with Tauri + React + Rust</span>
          </div>
          <div className="flex items-center gap-3">
            <Info size={16} className="text-primary" />
            <span>Client for secure, obfuscated proxy tunneling</span>
          </div>
        </div>
      </div>

      <div className="bg-card rounded-xl border border-border p-5">
        <h3 className="text-sm font-semibold mb-3">License</h3>
        <p className="text-xs text-muted-foreground">
          Proprietary. All rights reserved.
        </p>
      </div>
    </div>
  );
}
