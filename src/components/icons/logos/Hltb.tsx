interface HLTBIconProps {
  size?: number;
  className?: string;
}

export function HLTB({ size = 24, className = '' }: HLTBIconProps) {
  return (
    <svg
      role="img"
      viewBox="0 0 32 32"
      xmlns="http://www.w3.org/2000/svg"
      width={size}
      height={size}
      className={className}
      fill="currentColor"
    >
      <title>HowLongToBeat</title>
      <path
        fillRule="evenodd"
        clipRule="evenodd"
        d="M0 0 C10.56 0 21.12 0 32 0 C32 10.56 32 21.12 32 32 C21.44 32 10.88 32 0 32 C0 21.44 0 10.88 0 0 Z M11 6 C12.32 6 13.64 6 15 6 C15 9.96 15 13.92 15 18 C15.66 18 16.32 18 17 18 C17 14.04 17 10.08 17 6 C18.32 6 19.64 6 21 6 C21 12.6 21 19.2 21 26 C19.68 26 18.36 26 17 26 C17 24.68 17 23.36 17 22 C16.34 22 15.68 22 15 22 C15 23.32 15 24.64 15 26 C13.68 26 12.36 26 11 26 C11 19.4 11 12.8 11 6 Z"
      />
    </svg>
  );
}
