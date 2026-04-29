import type { ReactNode } from "react";

const routes = [
  {
    d: "M 170 210 C 220 238 236 334 306 362",
    delay: "-0.4s",
    duration: "4.4s",
  },
  {
    d: "M 416 362 C 462 326 506 326 550 362",
    delay: "-1.7s",
    duration: "5.4s",
  },
  {
    d: "M 416 362 C 466 362 500 362 550 362",
    delay: "-3s",
    duration: "4.9s",
  },
  {
    d: "M 416 362 C 462 398 506 398 550 362",
    delay: "-0.8s",
    duration: "4.8s",
  },
  {
    d: "M 660 362 C 708 326 748 326 796 362",
    delay: "-2.4s",
    duration: "5.1s",
  },
  {
    d: "M 660 362 C 710 362 746 362 796 362",
    delay: "-1.1s",
    duration: "4.6s",
  },
  {
    d: "M 660 362 C 708 398 748 398 796 362",
    delay: "-3.4s",
    duration: "5.6s",
  },
  {
    d: "M 906 362 C 956 326 906 232 944 200",
    delay: "-2.6s",
    duration: "5.8s",
  },
];

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="relay-flow__metric">
      <span>{value}</span>
      <small>{label}</small>
    </div>
  );
}

function Node({
  className,
  eyebrow,
  title,
  children,
}: {
  className: string;
  eyebrow: string;
  title: string;
  children: ReactNode;
}) {
  return (
    <div className={`relay-flow__node ${className}`}>
      <div className="relay-flow__icon">{children}</div>
      <span>{eyebrow}</span>
      <strong>{title}</strong>
    </div>
  );
}

function AwsCloudWatchIcon() {
  return (
    <svg aria-hidden="true" className="relay-flow__aws-icon" viewBox="0 0 48 48">
      <path
        d="M29.4998,17.0004 C26.2418,17.0004 23.4708,19.0924 22.4378,22.0004 L24.6068,22.0004 C25.5198,20.2224 27.3678,19.0004 29.4998,19.0004 C32.5318,19.0004 34.9998,21.4674 34.9998,24.5004 C34.9998,27.5324 32.5318,30.0004 29.4998,30.0004 C27.7958,30.0004 26.2708,29.2214 25.2608,28.0004 L22.8718,28.0004 C24.1318,30.3764 26.6288,32.0004 29.4998,32.0004 C33.6358,32.0004 36.9998,28.6364 36.9998,24.5004 C36.9998,20.3644 33.6358,17.0004 29.4998,17.0004 L29.4998,17.0004 Z M1.9998,33.0004 L14.9998,33.0004 L14.9998,31.0004 L1.9998,31.0004 L1.9998,33.0004 Z M1.9998,18.0004 L14.9998,18.0004 L14.9998,16.0004 L1.9998,16.0004 L1.9998,18.0004 Z M1.9998,26.0004 L28.9998,26.0004 L28.9998,24.0004 L1.9998,24.0004 L1.9998,26.0004 Z M43.8428,35.7384 L38.8178,31.2134 C38.3998,31.7924 37.9318,32.3304 37.4148,32.8214 L42.4118,37.3294 C42.8478,37.7224 43.5278,37.6874 43.9228,37.2494 C44.3148,36.8124 44.2798,36.1344 43.8428,35.7384 L43.8428,35.7384 Z M45.4088,38.5884 C44.8028,39.2594 43.9658,39.6004 43.1248,39.6004 C42.3938,39.6004 41.6588,39.3414 41.0738,38.8154 L35.8338,34.0894 C34.0168,35.2934 31.8398,36.0004 29.4998,36.0004 C24.3798,36.0004 20.0328,32.6344 18.5488,28.0004 L20.6768,28.0004 C22.0738,31.5094 25.4988,34.0004 29.4998,34.0004 C34.7378,34.0004 38.9998,29.7384 38.9998,24.5004 C38.9998,19.2624 34.7378,15.0004 29.4998,15.0004 C25.1278,15.0004 21.4458,17.9734 20.3448,22.0004 L18.2818,22.0004 C19.4278,16.8574 24.0188,13.0004 29.4998,13.0004 C35.8408,13.0004 40.9998,18.1584 40.9998,24.5004 C40.9998,26.2774 40.5828,27.9554 39.8588,29.4604 L45.1828,34.2544 C46.4368,35.3884 46.5378,37.3314 45.4088,38.5884 L45.4088,38.5884 Z M1.9998,41.0004 L20.9998,41.0004 L20.9998,39.0004 L1.9998,39.0004 L1.9998,41.0004 Z M1.9998,10.0004 L22.9998,10.0004 L22.9998,8.0004 L1.9998,8.0004 L1.9998,10.0004 Z"
        fill="#E7157B"
      />
    </svg>
  );
}

function AwsLambdaIcon() {
  return (
    <svg aria-hidden="true" className="relay-flow__aws-icon" viewBox="0 0 64 64">
      <rect width="64" height="64" fill="#ED7100" />
      <path
        d="M22.6794094,52 L13.5740701,52 L23.8376189,30.41 L28.3997494,39.861 L22.6794094,52 Z M24.7269406,27.667 C24.5606285,27.321 24.2120702,27.103 23.8306478,27.103 L23.8276601,27.103 C23.4452418,27.104 23.0966835,27.325 22.9323632,27.672 L11.0973143,52.569 C10.9499239,52.879 10.9708374,53.243 11.1540795,53.534 C11.3353298,53.824 11.6540117,54 11.9955989,54 L23.309802,54 C23.695208,54 24.0447622,53.777 24.2100784,53.428 L30.4044577,40.284 C30.5329264,40.01 30.5319305,39.692 30.3994783,39.42 L24.7269406,27.667 Z M51.0082382,52 L41.985557,52 L26.9547262,19.578 C26.7914017,19.226 26.4388599,19 26.0524581,19 L20.1279625,19 L20.1349337,12 L31.8146251,12 L46.7747483,44.42 C46.9380728,44.774 47.2906147,45 47.6790082,45 L51.0082382,45 L51.0082382,52 Z M52.0041191,43 L48.3143803,43 L33.354257,10.58 C33.1909326,10.226 32.8383907,10 32.450993,10 L19.1400486,10 C18.5913182,10 18.1451636,10.447 18.1441677,10.999 L18.1362006,19.999 C18.1362006,20.265 18.2407681,20.519 18.4269979,20.707 C18.6142235,20.895 18.8671772,21 19.1310857,21 L25.4170861,21 L40.4479168,53.422 C40.6112413,53.774 40.9627873,54 41.350185,54 L52.0041191,54 C52.5548412,54 53,53.552 53,53 L53,44 C53,43.448 52.5548412,43 52.0041191,43 L52.0041191,43 Z"
        fill="#fff"
      />
    </svg>
  );
}

function AwsKinesisIcon() {
  return (
    <svg aria-hidden="true" className="relay-flow__aws-icon" viewBox="0 0 64 64">
      <rect width="64" height="64" fill="#8C4FFF" />
      <g transform="translate(11 10)" fill="#fff">
        <path d="M33,39.001 L34,39.001 L34,38.001 L33,38.001 L33,39.001 Z M36,37.001 L36,40.001 C36,40.553 35.553,41.001 35,41.001 L32,41.001 C31.448,41.001 31,40.553 31,40.001 L31,37.001 C31,36.449 31.448,36.001 32,36.001 L35,36.001 C35.553,36.001 36,36.449 36,37.001 L36,37.001 Z M31,34.001 L34,34.001 L34,32.001 L31,32.001 L31,34.001 Z M37,31.001 L38,31.001 L38,30.001 L37,30.001 L37,31.001 Z M41,34.501 C41,33.145 40.633,31.874 40,30.775 L40,32.001 C40,32.553 39.553,33.001 39,33.001 L36,33.001 C35.448,33.001 35,32.553 35,32.001 L35,29.001 C35,28.449 35.448,28.001 36,28.001 L37.226,28.001 C36.127,27.369 34.857,27.001 33.5,27.001 C32.144,27.001 30.873,27.369 29.775,28.001 L34,28.001 L34,30.001 L27.514,30.001 C27.06,30.604 26.696,31.276 26.438,32.001 L30,32.001 L30,34.001 L26.026,34.001 C26.015,34.167 26,34.332 26,34.501 C26,35.015 26.053,35.516 26.152,36.001 L29,36.001 L29,38.001 L26.872,38.001 C27.059,38.353 27.276,38.685 27.514,39.001 L29,39.001 L29,40.487 C30.255,41.433 31.811,42.001 33.5,42.001 C34.857,42.001 36.127,41.633 37.226,41.001 L37,41.001 L37,39.001 L39.487,39.001 C39.725,38.685 39.942,38.353 40.128,38.001 L37,38.001 L37,36.001 L40.849,36.001 C40.948,35.516 41,35.015 41,34.501 L41,34.501 Z M43,34.501 C43,39.739 38.739,44.001 33.5,44.001 C28.262,44.001 24,39.739 24,34.501 C24,29.263 28.262,25.001 33.5,25.001 C38.739,25.001 43,29.263 43,34.501 L43,34.501 Z M26.646,25.601 C8.695,27.162 5,31.688 5,38 L7,38 C7,33.8 8.412,29.579 24.467,27.825 C25.088,26.986 25.821,26.238 26.646,25.601 L26.646,25.601 Z M10,43.876 L10,44 L12,44 L12,43.876 C11.999,39.591 12.006,35.169 22.402,32.742 C22.521,31.99 22.713,31.264 22.973,30.569 C10.006,33.284 9.999,39.146 10,43.876 L10,43.876 Z M2,13 L0,13 C0,16.614 2.747,20.285 19.012,22 C2.747,23.715 0,27.386 0,31 L2,31 C2,25.617 15.412,23 43,23 L43,21 C15.412,21 2,18.383 2,13 L2,13 Z M7,6 L5,6 C5,14.233 11.218,19 43,19 L43,17 C9.78,17 7,11.566 7,6 L7,6 Z M43,13 L43,15 C28.551,15 19.209,13.443 14.441,10.241 C9.998,7.257 9.999,3.302 9.99997,0.124 L9.99997,0 L12,0 L12,0.124 C11.999,3.244 11.999,6.191 15.556,8.58 C19.985,11.554 28.961,13 43,13 L43,13 Z" />
      </g>
    </svg>
  );
}

function OtlpIcon() {
  return (
    <svg aria-hidden="true" className="relay-flow__otlp-icon" viewBox="0 0 64 64">
      <rect width="64" height="64" rx="6" />
      <path d="M18 38V26M26 18h12M26 46h12M46 26v12" />
      <path d="M24 28l-6 4 6 4M40 28l6 4-6 4" />
    </svg>
  );
}

export function RelayFlowDiagram() {
  return (
    <div className="relay-flow" aria-label="Signals Relay routing diagram">
      <div className="relay-flow__glow" aria-hidden="true" />
      <div className="relay-flow__metrics" aria-hidden="true">
        <Metric label="log source" value="aws/spans" />
        <Metric label="buffer" value="trace keyed" />
        <Metric label="export" value="OTLP/HTTP" />
      </div>
      <div className="relay-flow__canvas">
        <svg
          aria-hidden="true"
          className="relay-flow__routes"
          fill="none"
          viewBox="0 0 1000 520"
          preserveAspectRatio="xMidYMid meet"
        >
          <defs>
            {routes.map(({ delay, duration }, index) => (
              <linearGradient
                gradientUnits="objectBoundingBox"
                id={`relay-flow-gradient-${index}`}
                key={`gradient-${index}`}
                x1="10%"
                x2="0%"
                y1="0%"
                y2="0%"
              >
                <animate
                  attributeName="x1"
                  begin={delay}
                  dur={duration}
                  repeatCount="indefinite"
                  values="10%;110%"
                />
                <animate
                  attributeName="x2"
                  begin={delay}
                  dur={duration}
                  repeatCount="indefinite"
                  values="0%;100%"
                />
                <stop offset="0%" stopColor="#38bdf8" stopOpacity="0" />
                <stop offset="24%" stopColor="#38bdf8" stopOpacity="0.92" />
                <stop offset="44%" stopColor="#a78bfa" />
                <stop offset="63%" stopColor="#fbbf24" stopOpacity="0.95" />
                <stop offset="100%" stopColor="#fbbf24" stopOpacity="0" />
              </linearGradient>
            ))}
          </defs>
          {routes.map(({ d }, index) => (
            <path className="relay-flow__route-base" d={d} key={`base-${index}`} />
          ))}
          {routes.map(({ d }, index) => (
            <path
              className="relay-flow__route-glow"
              d={d}
              key={`glow-${index}`}
              stroke={`url(#relay-flow-gradient-${index})`}
            />
          ))}
          {routes.map(({ d }, index) => (
            <path
              className="relay-flow__route-light"
              d={d}
              key={`light-${index}`}
              stroke={`url(#relay-flow-gradient-${index})`}
            />
          ))}
        </svg>
        <Node className="relay-flow__node--source" eyebrow="CloudWatch" title="aws/spans">
          <AwsCloudWatchIcon />
        </Node>
        <Node className="relay-flow__node--partitioner" eyebrow="Lambda" title="Partitioner">
          <AwsLambdaIcon />
        </Node>
        <Node className="relay-flow__node--stream" eyebrow="Kinesis" title="Trace lanes">
          <AwsKinesisIcon />
        </Node>
        <Node className="relay-flow__node--relay" eyebrow="Lambda" title="Relay">
          <AwsLambdaIcon />
        </Node>
        <Node className="relay-flow__node--otlp" eyebrow="Backend" title="OTLP">
          <OtlpIcon />
        </Node>
      </div>
    </div>
  );
}
