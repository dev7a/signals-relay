export function SchematicDiagram() {
  return (
    <div aria-label="Signals Relay trace pipeline schematic" className="schematic" role="img">
      <span className="schematic__tag">
        <b>FIG. 01</b> · trace pipeline · cross-section view
      </span>
      <div className="schematic__corner">
        <span>
          SCALE <b>1:1</b>
        </span>
        <span>
          SHEET <b>1 of 4</b>
        </span>
      </div>
      <span className="schematic__crosshair schematic__crosshair--tl" />
      <span className="schematic__crosshair schematic__crosshair--tr" />
      <span className="schematic__crosshair schematic__crosshair--bl" />
      <span className="schematic__crosshair schematic__crosshair--br" />

      <svg
        aria-hidden="true"
        preserveAspectRatio="xMidYMid meet"
        viewBox="0 0 660 520"
        xmlns="http://www.w3.org/2000/svg"
      >
        <defs>
          <pattern
            height="6"
            id="schematic-hatch"
            patternTransform="rotate(45)"
            patternUnits="userSpaceOnUse"
            width="6"
          >
            <line stroke="var(--signals-text)" strokeOpacity="0.22" strokeWidth="1" x1="0" x2="0" y1="0" y2="6" />
          </pattern>
          <linearGradient id="schematic-stream" x1="0" x2="1" y1="0" y2="0">
            <stop offset="0%" stopColor="var(--signals-text)" stopOpacity="0" />
            <stop offset="50%" stopColor="var(--signals-text)" stopOpacity="1" />
            <stop offset="100%" stopColor="var(--signals-text)" stopOpacity="0" />
          </linearGradient>
          <linearGradient id="schematic-rust" x1="0" x2="1" y1="0" y2="0">
            <stop offset="0%" stopColor="var(--signals-rust)" stopOpacity="0" />
            <stop offset="50%" stopColor="var(--signals-rust)" stopOpacity="1" />
            <stop offset="100%" stopColor="var(--signals-rust)" stopOpacity="0" />
          </linearGradient>
        </defs>

        <g stroke="var(--signals-text)" strokeOpacity="0.16" strokeWidth="1">
          <line strokeDasharray="2 3" x1="0" x2="660" y1="260" y2="260" />
          <line strokeDasharray="2 3" x1="330" x2="330" y1="0" y2="520" />
        </g>

        <g fill="var(--signals-gold)" fillOpacity="0.65" fontFamily="JetBrains Mono,monospace" fontSize="9">
          <text x="12" y="74">A</text>
          <text x="12" y="266">B</text>
          <text x="12" y="510">C</text>
          <text textAnchor="end" x="324" y="14">01</text>
          <text textAnchor="end" x="652" y="14">02</text>
        </g>

        <g className="schematic__hum">
          <rect fill="var(--signals-text)" fillOpacity="0.03" height="92" stroke="var(--signals-text)" strokeOpacity="0.6" strokeWidth="1.2" width="120" x="40" y="200" />
          <rect fill="url(#schematic-hatch)" height="84" opacity="0.55" width="112" x="44" y="204" />
          <text fill="var(--signals-text)" fontFamily="Instrument Serif,serif" fontSize="14" fontStyle="italic" textAnchor="middle" x="100" y="180">Source</text>
          <text fill="var(--signals-text)" fontFamily="JetBrains Mono,monospace" fontSize="10" textAnchor="middle" x="100" y="252">aws/spans</text>
          <text fill="var(--signals-text)" fillOpacity="0.55" fontFamily="JetBrains Mono,monospace" fontSize="8" textAnchor="middle" x="100" y="268">cloudwatch · logs</text>
        </g>

        <g>
          <rect fill="var(--signals-rust)" fillOpacity="0.06" height="60" stroke="var(--signals-rust)" strokeOpacity="0.75" strokeWidth="1.2" width="100" x="220" y="220" />
          <text fill="var(--signals-text)" fontFamily="Instrument Serif,serif" fontSize="13" fontStyle="italic" textAnchor="middle" x="270" y="248">Partitioner</text>
          <text fill="var(--signals-rust)" fontFamily="JetBrains Mono,monospace" fontSize="8" textAnchor="middle" x="270" y="266">λ · traceId</text>
        </g>

        <g>
          <rect fill="var(--signals-gold)" fillOpacity="0.05" height="180" stroke="var(--signals-gold)" strokeOpacity="0.65" strokeWidth="1.2" width="146" x="362" y="180" />
          <text fill="var(--signals-text)" fontFamily="Instrument Serif,serif" fontSize="13" fontStyle="italic" textAnchor="middle" x="435" y="172">Kinesis · trace lanes</text>
          <text fill="var(--signals-gold)" fontFamily="JetBrains Mono,monospace" fontSize="8" textAnchor="middle" x="435" y="376">60s tumbling window</text>

          <g fill="var(--signals-text)" fillOpacity="0.6" fontFamily="JetBrains Mono,monospace" fontSize="7">
            <line stroke="var(--signals-text)" strokeOpacity="0.2" x1="362" x2="508" y1="206" y2="206" />
            <text x="368" y="200">trace_id 7af2</text>
            <rect fill="var(--signals-text)" fillOpacity="0.8" height="6" width="14" x="372" y="210" />
            <rect fill="var(--signals-text)" fillOpacity="0.5" height="6" width="10" x="392" y="210" />
            <rect fill="var(--signals-text)" fillOpacity="0.85" height="6" width="22" x="408" y="210" />
            <rect fill="var(--signals-text)" fillOpacity="0.45" height="6" width="16" x="436" y="210" />
            <rect fill="var(--signals-text)" fillOpacity="0.65" height="6" width="34" x="458" y="210" />

            <line stroke="var(--signals-text)" strokeOpacity="0.2" x1="362" x2="508" y1="240" y2="240" />
            <text x="368" y="234">trace_id 9e3c</text>
            <rect fill="var(--signals-rust)" fillOpacity="0.78" height="6" width="22" x="372" y="244" />
            <rect fill="var(--signals-rust)" fillOpacity="0.55" height="6" width="14" x="400" y="244" />
            <rect fill="var(--signals-rust)" fillOpacity="0.9" height="6" width="40" x="420" y="244" />
            <rect fill="var(--signals-rust)" fillOpacity="0.6" height="6" width="20" x="466" y="244" />

            <line stroke="var(--signals-text)" strokeOpacity="0.2" x1="362" x2="508" y1="274" y2="274" />
            <text x="368" y="268">trace_id b80a</text>
            <rect fill="var(--signals-gold)" fillOpacity="0.78" height="6" width="12" x="372" y="278" />
            <rect fill="var(--signals-gold)" fillOpacity="0.92" height="6" width="18" x="390" y="278" />
            <rect fill="var(--signals-gold)" fillOpacity="0.55" height="6" width="26" x="414" y="278" />
            <rect fill="var(--signals-gold)" fillOpacity="0.72" height="6" width="20" x="446" y="278" />
            <rect fill="var(--signals-gold)" fillOpacity="0.4" height="6" width="14" x="472" y="278" />

            <line stroke="var(--signals-text)" strokeOpacity="0.2" x1="362" x2="508" y1="308" y2="308" />
            <text x="368" y="302">trace_id 1ce5</text>
            <rect fill="var(--signals-gold)" fillOpacity="0.78" height="6" width="30" x="372" y="312" />
            <rect fill="var(--signals-gold)" fillOpacity="0.55" height="6" width="14" x="408" y="312" />
            <rect fill="var(--signals-gold)" fillOpacity="0.85" height="6" width="22" x="428" y="312" />
            <rect fill="var(--signals-gold)" fillOpacity="0.6" height="6" width="36" x="456" y="312" />

            <line stroke="var(--signals-text)" strokeOpacity="0.2" x1="362" x2="508" y1="342" y2="342" />
            <text x="368" y="336">trace_id 4d11</text>
            <rect fill="var(--signals-text)" fillOpacity="0.7" height="6" width="18" x="372" y="346" />
            <rect fill="var(--signals-text)" fillOpacity="0.5" height="6" width="26" x="396" y="346" />
            <rect fill="var(--signals-text)" fillOpacity="0.8" height="6" width="40" x="428" y="346" />
          </g>

          <line stroke="var(--signals-gold)" strokeDasharray="3 3" strokeWidth="1.4" x1="498" x2="498" y1="180" y2="360" />
          <text fill="var(--signals-gold)" fontFamily="JetBrains Mono,monospace" fontSize="8" x="500" y="190">now</text>
        </g>

        <g>
          <rect fill="var(--signals-rust)" fillOpacity="0.06" height="60" stroke="var(--signals-rust)" strokeOpacity="0.75" strokeWidth="1.2" width="86" x="540" y="220" />
          <text fill="var(--signals-text)" fontFamily="Instrument Serif,serif" fontSize="13" fontStyle="italic" textAnchor="middle" x="583" y="248">Relay</text>
          <text fill="var(--signals-rust)" fontFamily="JetBrains Mono,monospace" fontSize="8" textAnchor="middle" x="583" y="266">λ · OTLP/HTTP</text>
        </g>

        <g className="schematic__hum">
          <rect fill="var(--signals-gold)" fillOpacity="0.07" height="76" stroke="var(--signals-gold)" strokeOpacity="0.8" strokeWidth="1.2" width="86" x="540" y="64" />
          <rect fill="url(#schematic-hatch)" height="68" opacity="0.4" width="78" x="544" y="68" />
          <g className="schematic__backend-dashboard" fill="none" strokeLinecap="round" strokeLinejoin="round">
            <rect fill="var(--signals-bg)" fillOpacity="0.46" height="34" stroke="var(--signals-gold)" strokeOpacity="0.9" strokeWidth="1.2" width="48" x="559" y="84" />
            <path d="M 559 94 L 607 94" stroke="var(--signals-gold)" strokeOpacity="0.5" strokeWidth="1" />
            <path d="M 568 110 L 568 104" stroke="var(--signals-text)" strokeOpacity="0.7" strokeWidth="2" />
            <path d="M 576 110 L 576 101" stroke="var(--signals-text)" strokeOpacity="0.85" strokeWidth="2" />
            <path d="M 584 110 L 584 106" stroke="var(--signals-text)" strokeOpacity="0.62" strokeWidth="2" />
            <path d="M 592 110 L 592 99" stroke="var(--signals-text)" strokeOpacity="0.9" strokeWidth="2" />
            <path d="M 566 90 L 574 86 L 584 89 L 595 84" stroke="var(--signals-rust)" strokeWidth="1.5" />
            <circle cx="566" cy="90" fill="var(--signals-rust)" r="1.8" stroke="none" />
            <circle cx="595" cy="84" fill="var(--signals-rust)" r="1.8" stroke="none" />
          </g>
          <text fill="var(--signals-text)" fontFamily="Instrument Serif,serif" fontSize="13" fontStyle="italic" textAnchor="middle" x="583" y="46">Backend</text>
          <text fill="var(--signals-gold)" fontFamily="JetBrains Mono,monospace" fontSize="8" textAnchor="middle" x="583" y="158">OTLP/HTTP</text>
        </g>

        <g fill="none" stroke="var(--signals-text)" strokeOpacity="0.22" strokeWidth="1.4">
          <path d="M 160 246 L 220 246" />
          <path d="M 320 246 C 338 246 344 212 362 212" />
          <path d="M 320 246 L 362 246" />
          <path d="M 320 246 C 338 246 344 280 362 280" />
          <path d="M 508 212 C 522 212 526 246 540 246" />
          <path d="M 508 246 L 540 246" />
          <path d="M 508 280 C 522 280 526 246 540 246" />
          <path d="M 583 220 L 583 140" />
        </g>

        <g className="schematic__flow" fill="none" strokeLinecap="round" strokeWidth="2">
          <path d="M 160 246 L 220 246" stroke="url(#schematic-stream)" strokeDasharray="14 60">
            <animate attributeName="stroke-dashoffset" dur="1.6s" from="74" repeatCount="indefinite" to="0" />
          </path>
          <path d="M 320 246 C 338 246 344 212 362 212" stroke="url(#schematic-stream)" strokeDasharray="8 40">
            <animate attributeName="stroke-dashoffset" dur="1.4s" from="50" repeatCount="indefinite" to="0" />
          </path>
          <path d="M 320 246 L 362 246" stroke="url(#schematic-stream)" strokeDasharray="8 40">
            <animate attributeName="stroke-dashoffset" begin="-0.3s" dur="1.4s" from="50" repeatCount="indefinite" to="0" />
          </path>
          <path d="M 320 246 C 338 246 344 280 362 280" stroke="url(#schematic-stream)" strokeDasharray="8 40">
            <animate attributeName="stroke-dashoffset" begin="-0.6s" dur="1.4s" from="50" repeatCount="indefinite" to="0" />
          </path>
          <path d="M 508 212 C 522 212 526 246 540 246" stroke="url(#schematic-rust)" strokeDasharray="7 30">
            <animate attributeName="stroke-dashoffset" dur="1.3s" from="38" repeatCount="indefinite" to="0" />
          </path>
          <path d="M 508 246 L 540 246" stroke="url(#schematic-rust)" strokeDasharray="7 30">
            <animate attributeName="stroke-dashoffset" begin="-0.3s" dur="1.3s" from="38" repeatCount="indefinite" to="0" />
          </path>
          <path d="M 508 280 C 522 280 526 246 540 246" stroke="url(#schematic-rust)" strokeDasharray="7 30">
            <animate attributeName="stroke-dashoffset" begin="-0.6s" dur="1.3s" from="38" repeatCount="indefinite" to="0" />
          </path>
          <path d="M 583 220 L 583 140" stroke="url(#schematic-rust)" strokeDasharray="14 60">
            <animate attributeName="stroke-dashoffset" dur="2.2s" from="74" repeatCount="indefinite" to="0" />
          </path>
        </g>

        <g className="schematic__flow">
          <circle fill="var(--signals-text)" r="3">
            <animateMotion dur="3s" path="M 160 246 L 220 246 L 320 246 C 338 246 344 212 362 212" repeatCount="indefinite" />
          </circle>
          <circle fill="var(--signals-text)" opacity="0.7" r="3">
            <animateMotion begin="-1s" dur="3s" path="M 160 246 L 220 246 L 320 246 L 362 246" repeatCount="indefinite" />
          </circle>
          <circle fill="var(--signals-text)" opacity="0.55" r="3">
            <animateMotion begin="-2s" dur="3s" path="M 160 246 L 220 246 L 320 246 C 338 246 344 280 362 280" repeatCount="indefinite" />
          </circle>
          <circle fill="var(--signals-rust)" r="3">
            <animateMotion dur="2.2s" path="M 508 212 C 522 212 526 246 540 246 L 583 246 L 583 140" repeatCount="indefinite" />
          </circle>
          <circle fill="var(--signals-rust)" opacity="0.7" r="3">
            <animateMotion begin="-1s" dur="2.2s" path="M 508 246 L 540 246 L 583 246 L 583 140" repeatCount="indefinite" />
          </circle>
          <circle fill="var(--signals-rust)" opacity="0.55" r="3">
            <animateMotion begin="-1.6s" dur="2.2s" path="M 508 280 C 522 280 526 246 540 246 L 583 246 L 583 140" repeatCount="indefinite" />
          </circle>
        </g>

        <g
          fill="var(--signals-text)"
          fillOpacity="0.55"
          fontFamily="JetBrains Mono,monospace"
          fontSize="8"
          stroke="var(--signals-text)"
          strokeOpacity="0.3"
          strokeWidth="1"
        >
          <line x1="40" x2="160" y1="448" y2="448" />
          <line x1="40" x2="40" y1="444" y2="452" />
          <line x1="160" x2="160" y1="444" y2="452" />
          <text stroke="none" textAnchor="middle" x="100" y="464">log group</text>

          <line x1="362" x2="508" y1="448" y2="448" />
          <line x1="362" x2="362" y1="444" y2="452" />
          <line x1="508" x2="508" y1="444" y2="452" />
          <text stroke="none" textAnchor="middle" x="435" y="464">trace-keyed lanes</text>
        </g>
      </svg>
    </div>
  );
}
