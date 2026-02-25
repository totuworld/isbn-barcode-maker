import { useEffect, useRef } from "react";

interface Props {
  epsContent: string;
}

/**
 * EPS 내용을 파싱해서 Canvas에 바코드를 렌더링하는 미리보기 컴포넌트.
 * 실제 EPS 렌더링 대신, EPS에서 바 좌표를 추출하여 Canvas에 그립니다.
 */
function BarcodePreview({ epsContent }: Props) {
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    // Parse BoundingBox for dimensions
    const bbMatch = epsContent.match(/%%HiResBoundingBox:\s*[\d.]+\s+[\d.]+\s+([\d.]+)\s+([\d.]+)/);
    if (!bbMatch) return;

    const epsWidth = parseFloat(bbMatch[1]);
    const epsHeight = parseFloat(bbMatch[2]);

    // Parse scale factor
    const scaleMatch = epsContent.match(/([\d.]+)\s+([\d.]+)\s+sc/);
    const scaleX = scaleMatch ? parseFloat(scaleMatch[1]) : 2.83464567;

    // Canvas display scale (render at 2x for sharpness)
    const displayScale = 3;
    const canvasW = Math.ceil(epsWidth * displayScale);
    const canvasH = Math.ceil(epsHeight * displayScale);

    canvas.width = canvasW;
    canvas.height = canvasH;
    canvas.style.width = `${Math.ceil(epsWidth * 1.5)}px`;
    canvas.style.height = `${Math.ceil(epsHeight * 1.5)}px`;

    // Clear
    ctx.fillStyle = "#ffffff";
    ctx.fillRect(0, 0, canvasW, canvasH);

    // Scale factor: mm coords * scaleX = pt, then * displayScale / 1 for canvas
    const s = scaleX * displayScale;

    ctx.fillStyle = "#000000";

    // Parse filled rectangles: n x1 y1 m x1 y2 l x2 y2 l x2 y1 l f c
    const barRegex = /n\s+([\d.]+)\s+([\d.]+)\s+m\s+([\d.]+)\s+([\d.]+)\s+l\s+([\d.]+)\s+([\d.]+)\s+l\s+([\d.]+)\s+([\d.]+)\s+l\s+f\s+c/g;
    let match;
    while ((match = barRegex.exec(epsContent)) !== null) {
      const x1 = parseFloat(match[1]) * s;
      const y1 = parseFloat(match[2]) * s;
      const x2 = parseFloat(match[5]) * s;
      const y2 = parseFloat(match[4]) * s;

      // EPS y-axis is bottom-up, canvas is top-down
      const cx = x1;
      const cy = canvasH - y2;
      const cw = x2 - x1;
      const ch = y2 - y1;

      ctx.fillRect(cx, cy, cw, ch);
    }

    // Parse text: n x y m (text) s c
    const textRegex = /n\s+([\d.]+)\s+([\d.]+)\s+m\s+\(([^)]+)\)\s+s\s+c/g;
    // Get font size from EPS
    const fontMatch = epsContent.match(/findfont\s+([\d.]+)\s+scalefont/);
    const fontSize = fontMatch ? parseFloat(fontMatch[1]) * s * 0.85 : 9 * displayScale;

    ctx.fillStyle = "#000000";
    ctx.font = `${fontSize}px Arial, Helvetica, sans-serif`;
    ctx.textBaseline = "bottom";

    while ((match = textRegex.exec(epsContent)) !== null) {
      const tx = parseFloat(match[1]) * s;
      const ty = parseFloat(match[2]) * s;
      const text = match[3];

      // EPS y-axis flip
      const cy = canvasH - ty;
      ctx.fillText(text, tx, cy);
    }
  }, [epsContent]);

  return <canvas ref={canvasRef} />;
}

export default BarcodePreview;
