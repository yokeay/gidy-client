const APP_VERSION = "v0.2.8";

export default function About() {
  return (
    <div style={{ display: "flex", flexDirection: "column", gap: 14, maxWidth: 680 }}>
      <div
        style={{
          background: "var(--card)",
          border: "1px solid var(--border)",
          borderRadius: 10,
          padding: "24px 28px",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 16, marginBottom: 20 }}>
          <div
            style={{
              width: 48,
              height: 48,
              borderRadius: 10,
              background: "var(--accent-green)",
              display: "flex",
              alignItems: "center",
              justifyContent: "center",
              fontSize: 24,
              fontWeight: 700,
              color: "#000",
              boxShadow: "0 0 16px rgba(46,204,113,0.4)",
            }}
          >
            G
          </div>
          <div>
            <div style={{ fontSize: 18, fontWeight: 700, color: "var(--fg)" }}>gidy-client</div>
            <div style={{ fontSize: 12, color: "var(--muted-fg)", fontFamily: "var(--font-mono)", marginTop: 2 }}>
              {APP_VERSION}
            </div>
          </div>
        </div>
        <div style={{ display: "flex", flexDirection: "column", gap: 12, fontSize: 13, color: "var(--muted-fg)" }}>
          <div style={{ display: "flex", alignItems: "flex-start", gap: 10 }}>
            <span style={{ color: "var(--accent-green)", flexShrink: 0 }}>⬡</span>
            <span>gidy protocol — QUIC-based secure proxy with protocol morphing</span>
          </div>
          <div style={{ display: "flex", alignItems: "flex-start", gap: 10 }}>
            <span style={{ color: "var(--accent-green)", flexShrink: 0 }}>⬡</span>
            <span>Built with Tauri + React + Rust</span>
          </div>
          <div style={{ display: "flex", alignItems: "flex-start", gap: 10 }}>
            <span style={{ color: "var(--accent-green)", flexShrink: 0 }}>⬡</span>
            <span>Client for secure, obfuscated proxy tunneling</span>
          </div>
        </div>
      </div>

      <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: 12 }}>
        <div
          style={{
            background: "var(--card)",
            border: "1px solid var(--border)",
            borderRadius: 10,
            padding: "20px 24px",
          }}
        >
          <div style={{ fontSize: 14, fontWeight: 500, color: "var(--fg)", marginBottom: 8 }}>License</div>
          <div style={{ fontSize: 12, color: "var(--muted-fg)" }}>Proprietary. All rights reserved.</div>
        </div>
        <a
          href="https://github.com/yokeay/gidy-client"
          target="_blank"
          rel="noreferrer"
          style={{
            background: "var(--card)",
            border: "1px solid var(--border)",
            borderRadius: 10,
            padding: "20px 24px",
            textDecoration: "none",
            transition: "border-color 0.15s",
            display: "block",
          }}
          onMouseEnter={e => ((e.currentTarget as HTMLElement).style.borderColor = "rgba(59,158,255,0.4)")}
          onMouseLeave={e => ((e.currentTarget as HTMLElement).style.borderColor = "var(--border)")}
        >
          <div style={{ display: "flex", alignItems: "center", justifyContent: "space-between", marginBottom: 8 }}>
            <div style={{ fontSize: 14, fontWeight: 500, color: "var(--fg)" }}>Repository</div>
            <span style={{ color: "var(--accent-blue)", fontSize: 14 }}>↗</span>
          </div>
          <div style={{ fontSize: 12, color: "var(--accent-blue)", fontFamily: "var(--font-mono)" }}>
            github.com/yokeay/gidy-client
          </div>
        </a>
      </div>
    </div>
  );
}
