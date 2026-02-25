import { useState, useCallback, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import BarcodePreview from "./components/BarcodePreview";

interface BarcodeResult {
  success: boolean;
  message: string;
  eps_content: string | null;
  file_path: string | null;
}

function App() {
  const [isbn, setIsbn] = useState("");
  const [addon, setAddon] = useState("");
  const [dpi, setDpi] = useState(600);
  const [barHeight, setBarHeight] = useState("15");
  const [barHeightNum, setBarHeightNum] = useState(15.0);
  const [barHeightAdjust, setBarHeightAdjust] = useState(8);
  const [adjustInput, setAdjustInput] = useState("0.8");
  const [epsContent, setEpsContent] = useState<string | null>(null);
  const [message, setMessage] = useState<{ text: string; type: "success" | "error" } | null>(null);
  const [isbnError, setIsbnError] = useState("");
  const [addonError, setAddonError] = useState("");

  const validateIsbn = (value: string) => {
    if (value.length === 0) { setIsbnError(""); return; }
    if (!/^\d*$/.test(value)) { setIsbnError("숫자만 입력 가능합니다"); return; }
    if (value.length !== 13) { setIsbnError("13자리를 입력해주세요"); return; }
    const digits = value.split("").map(Number);
    const sum = digits.reduce((acc, d, i) => acc + d * (i % 2 === 0 ? 1 : 3), 0);
    if (sum % 10 !== 0) { setIsbnError("체크디짓이 올바르지 않습니다"); return; }
    setIsbnError("");
  };

  const validateAddon = (value: string) => {
    if (value.length === 0) { setAddonError(""); return; }
    if (!/^\d*$/.test(value)) { setAddonError("숫자만 입력 가능합니다"); return; }
    if (value.length !== 5) { setAddonError("5자리를 입력해주세요"); return; }
    setAddonError("");
  };

  const handleIsbnChange = (value: string) => {
    const cleaned = value.replace(/\D/g, "").slice(0, 13);
    setIsbn(cleaned);
    validateIsbn(cleaned);
  };

  const handleAddonChange = (value: string) => {
    const cleaned = value.replace(/\D/g, "").slice(0, 5);
    setAddon(cleaned);
    validateAddon(cleaned);
  };

  const handleBarHeightBlur = () => {
    const num = parseFloat(barHeight);
    if (isNaN(num) || num < 5) {
      setBarHeight("5");
      setBarHeightNum(5);
    } else if (num > 50) {
      setBarHeight("50");
      setBarHeightNum(50);
    } else {
      setBarHeight(String(num));
      setBarHeightNum(num);
    }
  };

  const adjustMm = barHeightAdjust * 0.1;

  const updateAdjust = (val: number) => {
    const clamped = Math.max(-10, Math.min(10, val));
    setBarHeightAdjust(clamped);
    setAdjustInput((clamped * 0.1).toFixed(1));
  };

  const handleAdjustInputBlur = () => {
    const num = parseFloat(adjustInput);
    if (isNaN(num)) {
      setAdjustInput((barHeightAdjust * 0.1).toFixed(1));
    } else {
      const steps = Math.round(num * 10);
      updateAdjust(steps);
    }
  };

  const canGenerate = isbn.length === 13 && !isbnError && (addon.length === 0 || (addon.length === 5 && !addonError));

  const handleGenerate = useCallback(async () => {
    if (!canGenerate) return;
    try {
      const result = await invoke<BarcodeResult>("generate_barcode", {
        request: {
          isbn,
          addon,
          bar_height_mm: barHeightNum,
          dpi,
          addon_offset_mm: adjustMm,
        },
      });
      if (result.success && result.eps_content) {
        setEpsContent(result.eps_content);
        setMessage({ text: result.message, type: "success" });
      } else {
        setMessage({ text: result.message, type: "error" });
      }
    } catch (e) {
      setMessage({ text: `오류: ${e}`, type: "error" });
    }
  }, [isbn, addon, barHeightNum, dpi, adjustMm, canGenerate]);

  const handleSave = useCallback(async () => {
    if (!epsContent) return;
    const defaultName = addon
      ? `isbn_${isbn}_${addon}.eps`
      : `isbn_${isbn}.eps`;
    try {
      const filePath = await save({
        defaultPath: defaultName,
        filters: [{ name: "EPS", extensions: ["eps"] }],
      });
      if (filePath) {
        const result = await invoke<BarcodeResult>("save_eps", {
          content: epsContent,
          filePath,
        });
        setMessage({ text: result.message, type: result.success ? "success" : "error" });
      }
    } catch (e) {
      setMessage({ text: `저장 오류: ${e}`, type: "error" });
    }
  }, [epsContent, isbn, addon]);

  useEffect(() => {
    if (canGenerate) {
      handleGenerate();
    } else {
      setEpsContent(null);
    }
  }, [isbn, addon, barHeightNum, dpi, adjustMm]);

  return (
    <div className="app">

      <div className="card">
        <div className="section-title">바코드 정보</div>
        <div className="form-row">
          <div className="form-group" style={{ flex: 2 }}>
            <label>ISBN (13자리)</label>
            <input
              type="text"
              value={isbn}
              onChange={(e) => handleIsbnChange(e.target.value)}
              placeholder="9788969930460"
              className={isbnError ? "error" : ""}
              maxLength={13}
            />
            {isbnError && <span className="error-text">{isbnError}</span>}
          </div>
          <div className="form-group">
            <label>분류번호 (5자리)</label>
            <input
              type="text"
              value={addon}
              onChange={(e) => handleAddonChange(e.target.value)}
              placeholder="13590"
              className={addonError ? "error" : ""}
              maxLength={5}
            />
            {addonError && <span className="error-text">{addonError}</span>}
          </div>
        </div>
      </div>

      <div className="card">
        <div className="section-title">설정</div>
        <div className="form-row">
          <div className="form-group">
            <label>DPI</label>
            <select value={dpi} onChange={(e) => setDpi(Number(e.target.value))}>
              <option value={300}>300 DPI</option>
              <option value={600}>600 DPI</option>
              <option value={1200}>1200 DPI</option>
            </select>
          </div>
          <div className="form-group">
            <label>바코드 높이 (mm)</label>
            <input
              type="text"
              inputMode="decimal"
              value={barHeight}
              onChange={(e) => setBarHeight(e.target.value)}
              onBlur={handleBarHeightBlur}
              placeholder="15"
            />
            <span className="hint">5~50mm (기본 15mm)</span>
          </div>
        </div>
      </div>

      {addon.length === 5 && !addonError && (
        <div className="card">
          <div className="section-title">분류번호 위치 미세 조정</div>
          <div className="form-group">
            <label>분류번호 상하 오프셋 (mm)</label>
            <div className="slider-row">
              <button
                className="adj-btn"
                onClick={() => updateAdjust(barHeightAdjust - 1)}
              >−</button>
              <input
                type="range"
                min={-10}
                max={10}
                step={1}
                value={barHeightAdjust}
                onChange={(e) => updateAdjust(Number(e.target.value))}
              />
              <button
                className="adj-btn"
                onClick={() => updateAdjust(barHeightAdjust + 1)}
              >+</button>
              <input
                type="text"
                inputMode="decimal"
                className="adjust-input"
                value={adjustInput}
                onChange={(e) => setAdjustInput(e.target.value)}
                onBlur={handleAdjustInputBlur}
              />
              <span className="slider-unit">mm</span>
              {barHeightAdjust !== 0 && (
                <button className="reset-btn" onClick={() => updateAdjust(0)}>초기화</button>
              )}
            </div>
            <span className="hint">분류번호 전체(텍스트+바코드)를 위/아래로 이동 (±1.0mm)</span>
          </div>
        </div>
      )}

      <div className="card">
        <div className="section-title">미리보기</div>
        <div className="preview-area">
          {epsContent ? (
            <BarcodePreview epsContent={epsContent} />
          ) : (
            <span className="preview-placeholder">
              ISBN을 입력하면 바코드가 표시됩니다
            </span>
          )}
        </div>
        <div className="btn-row">
          <button className="primary" onClick={handleSave} disabled={!epsContent}>
            EPS 저장
          </button>
        </div>
      </div>

      {message && (
        <div className={`message ${message.type}`}>{message.text}</div>
      )}

      <div className="footer">사랑하는 arum이에게 totu가</div>
    </div>
  );
}

export default App;
