<script lang="ts">
  import { testAdaptiveTrigger, testVibration } from '../api';
  import Icon from '../Icon.svelte';

  type TesterTab = 'input' | 'calibration' | 'haptic' | 'trigger';
  type StickSide = 'left' | 'right';
  type StickDeadzone = { up: number; down: number; left: number; right: number };
  type StickSample = { x: number; y: number };
  type CalibrationPhase = 'idle' | 'neutral' | 'rotate' | 'done' | 'warning';

  interface FirmwareCapability {
    key: string;
    label: string;
    supported: boolean;
    reason: string;
  }

  interface CalibrationResult {
    phase: CalibrationPhase;
    progress: number;
    centerX: number;
    centerY: number;
    maxRadius: number;
    minX: number;
    maxX: number;
    minY: number;
    maxY: number;
    recommended: StickDeadzone;
    message: string;
  }

  let {
    text,
    deviceId = '',
    capabilities = [],
    onLog = () => {},
    onCalibrationApply = async () => {},
    onCalibrationClear = async () => {},
    isOpen = $bindable(false)
  }: {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    text: any;
    deviceId?: string;
    capabilities?: FirmwareCapability[];
    onLog?: (message: string, kind?: 'info' | 'error') => void;
    onCalibrationApply?: (side: StickSide, result: { centerX: number; centerY: number; deadzone: number; minX: number; maxX: number; minY: number; maxY: number }) => Promise<void> | void;
    onCalibrationClear?: (side: StickSide) => Promise<void> | void;
    isOpen: boolean;
  } = $props();

  const defaultDeadzone: StickDeadzone = { up: 8, down: 8, left: 8, right: 8 };
  const buttonLabels = [
    'SOUTH',
    'EAST',
    'WEST',
    'NORTH',
    'L1',
    'R1',
    'L2',
    'R2',
    'SHARE',
    'OPTIONS',
    'L3',
    'R3',
    'UP',
    'DOWN',
    'LEFT',
    'RIGHT',
    'PS',
    'TOUCHPAD'
  ];

  let activeTab = $state<TesterTab>('input');
  let gamepadIndex = $state<number | null>(null);
  let gamepadName = $state('');
  let isConnected = $state(false);
  let leftStickX = $state(0);
  let leftStickY = $state(0);
  let rightStickX = $state(0);
  let rightStickY = $state(0);
  let leftDeadzoneThreshold = $state<StickDeadzone>({ ...defaultDeadzone });
  let rightDeadzoneThreshold = $state<StickDeadzone>({ ...defaultDeadzone });
  let buttonsPressed = $state<boolean[]>(new Array(18).fill(false));
  let triggerValues = $state<{ L2: number; R2: number }>({ L2: 0, R2: 0 });
  let canvasL: HTMLCanvasElement | null = null;
  let canvasR: HTMLCanvasElement | null = null;
  let animationFrameId = 0;

  let weakMagnitude = $state(1.0);
  let strongMagnitude = $state(0.8);
  let vibrationDuration = $state(500);
  let isVibrating = $state(false);
  let vibrationError = $state('');
  let adaptiveTriggerStart = $state(0.25);
  let adaptiveTriggerStrength = $state(0.75);
  let adaptiveTriggerDuration = $state(700);
  let isAdaptiveTriggerTesting = $state(false);
  let adaptiveTriggerError = $state('');
  let calibratingSide = $state<StickSide | null>(null);
  let leftCalibration = $state<CalibrationResult>(emptyCalibration());
  let rightCalibration = $state<CalibrationResult>(emptyCalibration());

  let canUseBridge = $derived(Boolean(deviceId));
  let canTestVibration = $derived(canUseBridge && capabilities.find((capability) => capability.key === 'vibration')?.supported !== false);
  let canTestTrigger = $derived(canUseBridge && capabilities.find((capability) => capability.key === 'trigger')?.supported !== false);
  let leftDrift = $derived(getDrift(leftStickX, leftStickY));
  let rightDrift = $derived(getDrift(rightStickX, rightStickY));

  function emptyCalibration(): CalibrationResult {
    return {
      phase: 'idle',
      progress: 0,
      centerX: 0,
      centerY: 0,
      maxRadius: 0,
      minX: -1,
      maxX: 1,
      minY: -1,
      maxY: 1,
      recommended: { ...defaultDeadzone },
      message: ''
    };
  }

  function unsupportedReason(key: string) {
    return capabilities.find((capability) => capability.key === key && !capability.supported)?.reason ?? text.capRequiresBridge;
  }

  function setCanvasL(node: HTMLCanvasElement) {
    canvasL = node;
    drawStick(canvasL, 0, 0, leftDeadzoneThreshold);
  }

  function setCanvasR(node: HTMLCanvasElement) {
    canvasR = node;
    drawStick(canvasR, 0, 0, rightDeadzoneThreshold);
  }

  function getDrift(x: number, y: number) {
    return Math.min(100, Math.round(Math.sqrt(x * x + y * y) * 1000) / 10);
  }

  function getStick(side: StickSide): StickSample {
    return side === 'left' ? { x: leftStickX, y: leftStickY } : { x: rightStickX, y: rightStickY };
  }

  function sleep(ms: number) {
    return new Promise((resolve) => window.setTimeout(resolve, ms));
  }

  function pollGamepad() {
    if (!isOpen) {
      if (animationFrameId) cancelAnimationFrame(animationFrameId);
      return;
    }

    try {
      const gamepads = navigator.getGamepads();
      let activeGamepad: Gamepad | null = gamepadIndex !== null ? gamepads[gamepadIndex] : null;

      if (!activeGamepad) {
        for (let i = 0; i < gamepads.length; i += 1) {
          if (gamepads[i]) {
            activeGamepad = gamepads[i];
            gamepadIndex = i;
            break;
          }
        }
      }

      if (activeGamepad?.connected) {
        isConnected = true;
        gamepadName = activeGamepad.id;
        leftStickX = roundAxis(activeGamepad.axes[0] || 0);
        leftStickY = roundAxis(activeGamepad.axes[1] || 0);
        rightStickX = roundAxis(activeGamepad.axes[2] || 0);
        rightStickY = roundAxis(activeGamepad.axes[3] || 0);

        const nextButtons = new Array(18).fill(false);
        for (let i = 0; i < activeGamepad.buttons.length && i < 18; i += 1) {
          nextButtons[i] = activeGamepad.buttons[i].pressed;
        }
        buttonsPressed = nextButtons;
        triggerValues = {
          L2: activeGamepad.buttons[6]?.value ?? 0,
          R2: activeGamepad.buttons[7]?.value ?? 0
        };

        drawStick(canvasL, leftStickX, leftStickY, leftDeadzoneThreshold);
        drawStick(canvasR, rightStickX, rightStickY, rightDeadzoneThreshold);
      } else if (isConnected) {
        isConnected = false;
        gamepadIndex = null;
        resetInputs();
      }
    } catch {
      if (isConnected) {
        isConnected = false;
        resetInputs();
      }
    }

    animationFrameId = requestAnimationFrame(pollGamepad);
  }

  function roundAxis(value: number) {
    return Math.round(value * 1000) / 1000;
  }

  function resetInputs() {
    leftStickX = 0;
    leftStickY = 0;
    rightStickX = 0;
    rightStickY = 0;
    buttonsPressed = new Array(18).fill(false);
    triggerValues = { L2: 0, R2: 0 };
    drawStick(canvasL, 0, 0, leftDeadzoneThreshold);
    drawStick(canvasR, 0, 0, rightDeadzoneThreshold);
  }

  function isStickOverDeadzone(x: number, y: number, deadzone: StickDeadzone) {
    const percentX = x * 100;
    const percentY = y * 100;
    return (
      (percentX < 0 && Math.abs(percentX) > deadzone.left) ||
      (percentX > 0 && percentX > deadzone.right) ||
      (percentY < 0 && Math.abs(percentY) > deadzone.up) ||
      (percentY > 0 && percentY > deadzone.down)
    );
  }

  function drawStick(canvas: HTMLCanvasElement | null, x: number, y: number, deadzoneThreshold: StickDeadzone) {
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const styles = getComputedStyle(canvas);
    const cssColor = (name: string, fallback: string) => styles.getPropertyValue(name).trim() || fallback;
    const size = canvas.width;
    const center = size / 2;
    const radius = size / 2 - 8;
    const exceeded = isStickOverDeadzone(x, y, deadzoneThreshold);
    const posX = center + x * radius;
    const posY = center + y * radius;

    ctx.clearRect(0, 0, size, size);
    ctx.beginPath();
    ctx.arc(center, center, radius, 0, Math.PI * 2);
    ctx.fillStyle = cssColor('--tester-canvas-bg', 'rgba(0, 0, 0, 0.25)');
    ctx.fill();
    ctx.lineWidth = 1;
    ctx.strokeStyle = cssColor('--tester-canvas-line', 'rgba(255, 255, 255, 0.1)');
    ctx.stroke();

    ctx.beginPath();
    ctx.moveTo(center, 8);
    ctx.lineTo(center, size - 8);
    ctx.moveTo(8, center);
    ctx.lineTo(size - 8, center);
    ctx.strokeStyle = cssColor('--tester-canvas-guide', 'rgba(255, 255, 255, 0.08)');
    ctx.stroke();

    const upY = center - radius * (deadzoneThreshold.up / 100);
    const downY = center + radius * (deadzoneThreshold.down / 100);
    const leftX = center - radius * (deadzoneThreshold.left / 100);
    const rightX = center + radius * (deadzoneThreshold.right / 100);
    ctx.beginPath();
    ctx.moveTo(center, upY);
    ctx.lineTo(rightX, center);
    ctx.lineTo(center, downY);
    ctx.lineTo(leftX, center);
    ctx.closePath();
    ctx.fillStyle = cssColor('--tester-canvas-deadzone', 'rgba(255, 255, 255, 0.03)');
    ctx.fill();
    ctx.setLineDash([3, 3]);
    ctx.strokeStyle = 'rgba(255, 99, 132, 0.15)';
    ctx.stroke();
    ctx.setLineDash([]);

    ctx.beginPath();
    ctx.moveTo(center, center);
    ctx.lineTo(posX, posY);
    ctx.lineWidth = 1.5;
    ctx.strokeStyle = exceeded ? 'rgba(255, 171, 0, 0.4)' : 'rgba(99, 226, 183, 0.4)';
    ctx.stroke();

    ctx.beginPath();
    ctx.arc(posX, posY, 6, 0, Math.PI * 2);
    ctx.fillStyle = exceeded ? '#ffab00' : '#63e2b7';
    ctx.shadowBlur = 8;
    ctx.shadowColor = exceeded ? '#ffab00' : '#63e2b7';
    ctx.fill();
    ctx.shadowBlur = 0;
  }

  async function collectSamples(side: StickSide, durationMs: number, onProgress: (progress: number) => void) {
    const startedAt = performance.now();
    const samples: StickSample[] = [];
    while (performance.now() - startedAt < durationMs) {
      samples.push(getStick(side));
      onProgress(Math.min(100, ((performance.now() - startedAt) / durationMs) * 100));
      await sleep(50);
    }
    onProgress(100);
    return samples;
  }

  function average(samples: StickSample[]) {
    const sum = samples.reduce((acc, sample) => ({ x: acc.x + sample.x, y: acc.y + sample.y }), { x: 0, y: 0 });
    return { x: sum.x / Math.max(1, samples.length), y: sum.y / Math.max(1, samples.length) };
  }

  function analyzeCalibration(neutralSamples: StickSample[], rotateSamples: StickSample[]) {
    const center = average(neutralSamples);
    const centeredNeutral = neutralSamples.map((sample) => Math.hypot(sample.x - center.x, sample.y - center.y));
    const neutralNoisePercent = Math.max(0, ...centeredNeutral) * 100;
    const minX = Math.min(...rotateSamples.map((sample) => sample.x));
    const maxX = Math.max(...rotateSamples.map((sample) => sample.x));
    const minY = Math.min(...rotateSamples.map((sample) => sample.y));
    const maxY = Math.max(...rotateSamples.map((sample) => sample.y));
    const maxRadius = Math.max(...rotateSamples.map((sample) => Math.hypot(sample.x - center.x, sample.y - center.y))) * 100;
    const base = Math.min(8, Math.max(1, neutralNoisePercent + 0.4));
    const recommended = {
      up: roundPercent(Math.max(base, Math.abs(center.y - minY) < 0.3 ? 8 : base)),
      down: roundPercent(Math.max(base, Math.abs(maxY - center.y) < 0.3 ? 8 : base)),
      left: roundPercent(Math.max(base, Math.abs(center.x - minX) < 0.3 ? 8 : base)),
      right: roundPercent(Math.max(base, Math.abs(maxX - center.x) < 0.3 ? 8 : base))
    };
    const warning = neutralNoisePercent + 0.4 > 8 || maxRadius < 60;

    return {
      phase: warning ? 'warning' : 'done',
      progress: 100,
      centerX: center.x,
      centerY: center.y,
      maxRadius,
      minX,
      maxX,
      minY,
      maxY,
      recommended,
      message: warning ? text.calibrationWarning : text.calibrationDone
    } satisfies CalibrationResult;
  }

  function roundPercent(value: number) {
    return Math.round(value * 10) / 10;
  }

  async function startCalibration(side: StickSide) {
    if (!isConnected || calibratingSide) return;

    calibratingSide = side;
    const setResult = (result: CalibrationResult) => {
      if (side === 'left') leftCalibration = result;
      else rightCalibration = result;
    };

    try {
      setResult({ ...emptyCalibration(), phase: 'neutral', message: text.calibrationNeutral });
      const neutralSamples = await collectSamples(side, 2000, (progress) => {
        setResult({ ...emptyCalibration(), phase: 'neutral', progress, message: text.calibrationNeutral });
      });

      setResult({ ...emptyCalibration(), phase: 'rotate', message: text.calibrationRotate });
      const rotateSamples = await collectSamples(side, 5000, (progress) => {
        setResult({ ...emptyCalibration(), phase: 'rotate', progress, message: text.calibrationRotate });
      });

      const result = analyzeCalibration(neutralSamples, rotateSamples);
      setResult(result);
      if (side === 'left') leftDeadzoneThreshold = result.recommended;
      else rightDeadzoneThreshold = result.recommended;
      onLog(`${side === 'left' ? text.leftStick : text.rightStick}: ${result.message}`);
      if (result.phase === 'done') {
        await onCalibrationApply(side, {
          centerX: result.centerX,
          centerY: result.centerY,
          minX: result.minX,
          maxX: result.maxX,
          minY: result.minY,
          maxY: result.maxY,
          deadzone: Math.max(
            result.recommended.up,
            result.recommended.down,
            result.recommended.left,
            result.recommended.right
          )
        });
      } else {
        onLog(text.calibrationNotSaved, 'error');
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : text.calibrationFailed;
      setResult({ ...emptyCalibration(), phase: 'warning', message });
      onLog(message, 'error');
    } finally {
      calibratingSide = null;
    }
  }

  function resetCalibration(side: StickSide) {
    if (side === 'left') {
      leftDeadzoneThreshold = { ...defaultDeadzone };
      leftCalibration = emptyCalibration();
    } else {
      rightDeadzoneThreshold = { ...defaultDeadzone };
      rightCalibration = emptyCalibration();
    }
  }

  async function clearCalibration(side: StickSide) {
    resetCalibration(side);
    await onCalibrationClear(side);
  }

  async function triggerVibration(overrideWeak?: number, overrideStrong?: number, overrideDuration?: number) {
    if (!canTestVibration || isVibrating) return;

    try {
      vibrationError = '';
      isVibrating = true;
      await testVibration(
        deviceId,
        overrideWeak ?? weakMagnitude,
        overrideStrong ?? strongMagnitude,
        overrideDuration ?? vibrationDuration
      );
    } catch (error) {
      vibrationError = error instanceof Error ? error.message : text.vibrationTestFailed;
      onLog(vibrationError, 'error');
    } finally {
      isVibrating = false;
    }
  }

  async function triggerDoublePulse() {
    if (!canTestVibration || isVibrating) return;

    isVibrating = true;
    try {
      vibrationError = '';
      await testVibration(deviceId, 0, 1, 250);
      await sleep(350);
      await testVibration(deviceId, 0, 1, 250);
    } catch (error) {
      vibrationError = error instanceof Error ? error.message : text.vibrationTestFailed;
      onLog(vibrationError, 'error');
    } finally {
      isVibrating = false;
    }
  }

  async function triggerAdaptiveTrigger(side: 'left' | 'right') {
    if (!canTestTrigger || isAdaptiveTriggerTesting) return;

    try {
      adaptiveTriggerError = '';
      isAdaptiveTriggerTesting = true;
      await testAdaptiveTrigger(deviceId, side, adaptiveTriggerStart, adaptiveTriggerStrength, adaptiveTriggerDuration);
    } catch (error) {
      adaptiveTriggerError = error instanceof Error ? error.message : text.adaptiveTriggerFailed;
      onLog(adaptiveTriggerError, 'error');
    } finally {
      isAdaptiveTriggerTesting = false;
    }
  }

  $effect(() => {
    if (isOpen) {
      window.addEventListener('gamepadconnected', handleConnected);
      window.addEventListener('gamepaddisconnected', handleDisconnected);
      resetInputs();
      animationFrameId = requestAnimationFrame(pollGamepad);
      const timer = window.setTimeout(() => {
        drawStick(canvasL, 0, 0, leftDeadzoneThreshold);
        drawStick(canvasR, 0, 0, rightDeadzoneThreshold);
      }, 50);

      return () => {
        window.removeEventListener('gamepadconnected', handleConnected);
        window.removeEventListener('gamepaddisconnected', handleDisconnected);
        if (animationFrameId) cancelAnimationFrame(animationFrameId);
        window.clearTimeout(timer);
      };
    }
  });

  function handleConnected(event: GamepadEvent) {
    gamepadIndex = event.gamepad.index;
    isConnected = true;
    gamepadName = event.gamepad.id;
    resetInputs();
  }

  function handleDisconnected(event: GamepadEvent) {
    if (gamepadIndex === event.gamepad.index) {
      gamepadIndex = null;
      isConnected = false;
      resetInputs();
    }
  }
</script>

{#if isOpen}
  <div class="tester-modal-overlay">
    <div class="tester-modal" onclick={(event) => event.stopPropagation()} onkeydown={(event) => event.stopPropagation()} role="dialog" aria-modal="true" tabindex="-1">
      <div class="tester-head">
        <div>
          <h3><Icon name="gamepad" size={18} /> {text.inputTester}</h3>
          <p>{isConnected ? text.connectedGamepad : text.noGamepad}</p>
        </div>
        <button type="button" onclick={() => (isOpen = false)} aria-label={text.close}>×</button>
      </div>

      <div class="tester-tabs" role="tablist">
        <button class:active={activeTab === 'input'} type="button" onclick={() => (activeTab = 'input')}>{text.inputTab}</button>
        <button class:active={activeTab === 'calibration'} type="button" onclick={() => (activeTab = 'calibration')}>{text.stickCalibration}</button>
        <button class:active={activeTab === 'haptic'} type="button" onclick={() => (activeTab = 'haptic')}>{text.hapticTab}</button>
        <button class:active={activeTab === 'trigger'} type="button" onclick={() => (activeTab = 'trigger')}>{text.triggerTab}</button>
      </div>

      <div class="connection-strip">
        <span class:online={isConnected}></span>
        <strong>{isConnected ? text.connectedGamepad : text.noGamepad}</strong>
        {#if gamepadName}
          <code>{gamepadName}</code>
        {/if}
      </div>

      {#if activeTab === 'input'}
        <div class="sticks-container">
          <section class="stick-card">
            <strong><span></span>{text.leftStick}</strong>
            <canvas use:setCanvasL width="112" height="112"></canvas>
            <div class="stick-values">
              <span>X {leftStickX >= 0 ? '+' : ''}{leftStickX.toFixed(3)}</span>
              <span>Y {leftStickY >= 0 ? '+' : ''}{leftStickY.toFixed(3)}</span>
              <b class:warn={isStickOverDeadzone(leftStickX, leftStickY, leftDeadzoneThreshold)}>{text.drift} {leftDrift.toFixed(1)}%</b>
            </div>
          </section>
          <section class="stick-card">
            <strong><span></span>{text.rightStick}</strong>
            <canvas use:setCanvasR width="112" height="112"></canvas>
            <div class="stick-values">
              <span>X {rightStickX >= 0 ? '+' : ''}{rightStickX.toFixed(3)}</span>
              <span>Y {rightStickY >= 0 ? '+' : ''}{rightStickY.toFixed(3)}</span>
              <b class:warn={isStickOverDeadzone(rightStickX, rightStickY, rightDeadzoneThreshold)}>{text.drift} {rightDrift.toFixed(1)}%</b>
            </div>
          </section>
        </div>

        <section class="tester-card compact">
          <strong><Icon name="sliders" size={14} /> {text.inputAndTriggerDetector}</strong>
          <div class="trigger-row">
            <label>L2 <meter min="0" max="1" value={triggerValues.L2}></meter><span>{Math.round(triggerValues.L2 * 100)}%</span></label>
            <label>R2 <meter min="0" max="1" value={triggerValues.R2}></meter><span>{Math.round(triggerValues.R2 * 100)}%</span></label>
          </div>
          <div class="keys-display">
            {#each buttonLabels as label, index}
              <span class:active={buttonsPressed[index]}>{label}</span>
            {/each}
          </div>
        </section>
      {:else if activeTab === 'calibration'}
        <div class="calibration-grid">
          <section class="tester-card">
            <strong>{text.leftStick}</strong>
            <p>{leftCalibration.message || text.calibrationReady}</p>
            <progress max="100" value={leftCalibration.progress}></progress>
            <div class="result-grid">
              <span>{text.calibrationCenter}</span><b>{leftCalibration.centerX.toFixed(3)}, {leftCalibration.centerY.toFixed(3)}</b>
              <span>{text.calibrationRadius}</span><b>{leftCalibration.maxRadius.toFixed(1)}%</b>
              <span>{text.calibrationReach}</span><b>X {leftCalibration.minX.toFixed(2)}~{leftCalibration.maxX.toFixed(2)} / Y {leftCalibration.minY.toFixed(2)}~{leftCalibration.maxY.toFixed(2)}</b>
              <span>{text.deadzone}</span><b>{leftDeadzoneThreshold.up.toFixed(1)}%</b>
            </div>
            <div class="button-row">
              <button type="button" disabled={!isConnected || Boolean(calibratingSide)} onclick={() => startCalibration('left')}>{text.startCalibration}</button>
              <button type="button" disabled={Boolean(calibratingSide)} onclick={() => resetCalibration('left')}>{text.remeasure}</button>
              <button type="button" disabled={Boolean(calibratingSide)} onclick={() => clearCalibration('left')}>{text.clearCalibration}</button>
            </div>
          </section>
          <section class="tester-card">
            <strong>{text.rightStick}</strong>
            <p>{rightCalibration.message || text.calibrationReady}</p>
            <progress max="100" value={rightCalibration.progress}></progress>
            <div class="result-grid">
              <span>{text.calibrationCenter}</span><b>{rightCalibration.centerX.toFixed(3)}, {rightCalibration.centerY.toFixed(3)}</b>
              <span>{text.calibrationRadius}</span><b>{rightCalibration.maxRadius.toFixed(1)}%</b>
              <span>{text.calibrationReach}</span><b>X {rightCalibration.minX.toFixed(2)}~{rightCalibration.maxX.toFixed(2)} / Y {rightCalibration.minY.toFixed(2)}~{rightCalibration.maxY.toFixed(2)}</b>
              <span>{text.deadzone}</span><b>{rightDeadzoneThreshold.up.toFixed(1)}%</b>
            </div>
            <div class="button-row">
              <button type="button" disabled={!isConnected || Boolean(calibratingSide)} onclick={() => startCalibration('right')}>{text.startCalibration}</button>
              <button type="button" disabled={Boolean(calibratingSide)} onclick={() => resetCalibration('right')}>{text.remeasure}</button>
              <button type="button" disabled={Boolean(calibratingSide)} onclick={() => clearCalibration('right')}>{text.clearCalibration}</button>
            </div>
          </section>
        </div>
      {:else if activeTab === 'haptic'}
        <section class="tester-card">
          <strong><Icon name="zap" size={14} /> {text.hapticTest}</strong>
          <p>{canTestVibration ? text.hapticTestDesc : unsupportedReason('vibration')}</p>
          <div class:disabled={!canTestVibration} class="control-stack">
            <label>{text.strongVibration}<input type="range" min="0" max="1" step="0.05" bind:value={strongMagnitude} disabled={!canTestVibration || isVibrating} /><span>{Math.round(strongMagnitude * 100)}%</span></label>
            <label>{text.weakVibration}<input type="range" min="0" max="1" step="0.05" bind:value={weakMagnitude} disabled={!canTestVibration || isVibrating} /><span>{Math.round(weakMagnitude * 100)}%</span></label>
            <label>{text.vibrationDuration}<input type="number" min="100" max="3000" step="50" bind:value={vibrationDuration} disabled={!canTestVibration || isVibrating} /><span>ms</span></label>
          </div>
          <div class="button-row">
            <button type="button" disabled={!canTestVibration || isVibrating} onclick={() => triggerVibration(0, 1, 150)}>{text.tap}</button>
            <button type="button" disabled={!canTestVibration || isVibrating} onclick={() => triggerVibration(0.8, 0.8, 1000)}>{text.pulse}</button>
            <button type="button" disabled={!canTestVibration || isVibrating} onclick={triggerDoublePulse}>{text.doublePulse}</button>
            <button class="primary" type="button" disabled={!canTestVibration || isVibrating} onclick={() => triggerVibration()}>{isVibrating ? text.vibrationRunning : text.testVibration}</button>
          </div>
          {#if vibrationError}
            <div class="error-box">{vibrationError}</div>
          {/if}
        </section>
      {:else}
        <section class="tester-card">
          <strong><Icon name="sliders" size={14} /> {text.adaptiveTriggerTest}</strong>
          <p>{canTestTrigger ? text.adaptiveTriggerTestDesc : unsupportedReason('trigger')}</p>
          <div class:disabled={!canTestTrigger} class="control-stack">
            <label>{text.triggerStart}<input type="range" min="0" max="1" step="0.05" bind:value={adaptiveTriggerStart} disabled={!canTestTrigger || isAdaptiveTriggerTesting} /><span>{Math.round(adaptiveTriggerStart * 100)}%</span></label>
            <label>{text.triggerStrength}<input type="range" min="0" max="1" step="0.05" bind:value={adaptiveTriggerStrength} disabled={!canTestTrigger || isAdaptiveTriggerTesting} /><span>{Math.round(adaptiveTriggerStrength * 100)}%</span></label>
            <label>{text.vibrationDuration}<input type="number" min="100" max="3000" step="50" bind:value={adaptiveTriggerDuration} disabled={!canTestTrigger || isAdaptiveTriggerTesting} /><span>ms</span></label>
          </div>
          <div class="button-row">
            <button type="button" disabled={!canTestTrigger || isAdaptiveTriggerTesting} onclick={() => triggerAdaptiveTrigger('left')}>{text.testL2}</button>
            <button class="primary" type="button" disabled={!canTestTrigger || isAdaptiveTriggerTesting} onclick={() => triggerAdaptiveTrigger('right')}>{isAdaptiveTriggerTesting ? text.adaptiveTriggerRunning : text.testR2}</button>
          </div>
          {#if adaptiveTriggerError}
            <div class="error-box">{adaptiveTriggerError}</div>
          {/if}
        </section>
      {/if}
    </div>
  </div>
{/if}

<style>
  .tester-modal-overlay {
    position: fixed;
    inset: 0;
    z-index: 9999;
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(0, 0, 0, 0.65);
    backdrop-filter: blur(5px);
  }

  .tester-modal {
    width: min(1064px, calc(100vw - 72px));
    max-height: calc(100vh - 160px);
    box-sizing: border-box;
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px 14px;
    border: 1px solid var(--border);
    border-radius: 14px;
    background: var(--tester-bg);
    color: var(--tester-text);
    box-shadow: 0 18px 50px rgba(0, 0, 0, 0.3);
  }

  .tester-head,
  .connection-strip,
  .tester-tabs,
  .button-row,
  .trigger-row {
    display: flex;
    align-items: center;
  }

  .tester-head {
    justify-content: space-between;
    gap: 12px;
    padding-bottom: 7px;
    border-bottom: 1px solid var(--border);
  }

  .tester-head h3 {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 0;
    font-size: 1rem;
  }

  .tester-head p,
  .tester-card p {
    margin: 2px 0 0;
    color: var(--tester-muted);
    font-size: 0.72rem;
    line-height: 1.35;
  }

  .tester-head button {
    border: 0;
    background: transparent;
    color: var(--tester-muted);
    font-size: 1.4rem;
    cursor: pointer;
  }

  .tester-tabs {
    gap: 6px;
    padding: 3px;
    border: 1px solid var(--border);
    border-radius: 9px;
    background: var(--tester-soft);
  }

  .tester-tabs button {
    flex: 1;
    height: 30px;
    border: 0;
    border-radius: 7px;
    background: transparent;
    color: var(--tester-muted);
    font-weight: 600;
    cursor: pointer;
  }

  .tester-tabs button.active {
    background: var(--control-2);
    color: var(--tester-text);
    box-shadow: 0 1px 5px rgba(0, 0, 0, 0.16);
  }

  .connection-strip {
    gap: 8px;
    min-height: 30px;
    padding: 6px 10px;
    border: 1px solid var(--tester-card-border);
    border-radius: 8px;
    background: var(--tester-card);
    font-size: 0.76rem;
  }

  .connection-strip > span {
    width: 7px;
    height: 7px;
    border-radius: 50%;
    background: var(--tester-muted);
  }

  .connection-strip > span.online {
    background: #63e2b7;
    box-shadow: 0 0 8px rgba(99, 226, 183, 0.8);
  }

  .connection-strip code {
    margin-left: auto;
    max-width: 560px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    color: var(--tester-muted);
    font-size: 0.68rem;
  }

  .sticks-container,
  .calibration-grid {
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 10px;
  }

  .stick-card,
  .tester-card {
    min-width: 0;
    padding: 10px;
    border: 1px solid var(--tester-card-border);
    border-radius: 10px;
    background: var(--tester-card);
  }

  .stick-card {
    display: grid;
    grid-template-columns: auto minmax(0, 1fr);
    grid-template-rows: auto auto;
    align-items: center;
    gap: 6px 12px;
  }

  .stick-card strong,
  .tester-card > strong {
    display: flex;
    align-items: center;
    gap: 7px;
    color: var(--tester-text);
    font-size: 0.82rem;
  }

  .stick-card strong {
    grid-column: 1 / -1;
    justify-content: center;
  }

  .stick-card strong span {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: #63e2b7;
  }

  .stick-card canvas {
    width: 112px;
    height: 112px;
  }

  .stick-values,
  .result-grid {
    display: grid;
    gap: 5px;
    font-family: monospace;
    font-size: 0.72rem;
  }

  .stick-values b {
    color: #63e2b7;
  }

  .stick-values b.warn {
    color: #ffab00;
  }

  .tester-card.compact {
    margin-top: 10px;
  }

  .trigger-row {
    gap: 12px;
    margin-top: 8px;
  }

  .trigger-row label {
    flex: 1;
    display: grid;
    grid-template-columns: 28px 1fr 38px;
    align-items: center;
    gap: 8px;
    font-size: 0.72rem;
  }

  meter {
    width: 100%;
    height: 7px;
  }

  .keys-display {
    display: flex;
    flex-wrap: wrap;
    gap: 5px;
    margin-top: 8px;
  }

  .keys-display span {
    padding: 5px 8px;
    border: 1px solid var(--tester-card-border);
    border-radius: 5px;
    background: var(--tester-chip-bg);
    color: var(--tester-muted);
    font-size: 0.68rem;
    font-weight: 600;
  }

  .keys-display span.active {
    border-color: rgba(99, 226, 183, 0.35);
    background: rgba(99, 226, 183, 0.15);
    color: #63e2b7;
  }

  progress {
    width: 100%;
    height: 8px;
    margin: 10px 0;
  }

  .result-grid {
    grid-template-columns: auto minmax(0, 1fr);
    margin-bottom: 10px;
  }

  .result-grid span {
    color: var(--tester-muted);
  }

  .control-stack {
    display: grid;
    gap: 10px;
    margin: 12px 0;
  }

  .control-stack.disabled {
    opacity: 0.45;
    pointer-events: none;
  }

  .control-stack label {
    display: grid;
    grid-template-columns: 140px minmax(0, 1fr) 48px;
    align-items: center;
    gap: 10px;
    font-size: 0.76rem;
    color: var(--tester-muted);
  }

  .control-stack input[type='number'] {
    width: 78px;
  }

  .button-row {
    gap: 7px;
    flex-wrap: wrap;
  }

  .button-row button {
    min-height: 30px;
    padding: 6px 11px;
    border: 1px solid var(--tester-card-border);
    border-radius: 6px;
    background: var(--tester-chip-bg);
    color: var(--tester-text);
    font-size: 0.74rem;
    font-weight: 700;
    cursor: pointer;
  }

  .button-row button.primary {
    border-color: rgba(16, 185, 129, 0.3);
    background: rgba(16, 185, 129, 0.12);
    color: #10b981;
  }

  .button-row button:disabled {
    opacity: 0.45;
    cursor: not-allowed;
  }

  .error-box {
    margin-top: 10px;
    padding: 8px 10px;
    border: 1px solid rgba(255, 77, 79, 0.22);
    border-radius: 8px;
    background: rgba(255, 77, 79, 0.08);
    color: #ff8a8a;
    font-size: 0.72rem;
    line-height: 1.4;
  }

  @media (max-height: 690px) {
    .tester-modal {
      width: min(1064px, calc(100vw - 48px));
      padding: 10px 12px;
      gap: 6px;
    }

    .stick-card canvas {
      width: 96px;
      height: 96px;
    }

    .tester-card,
    .stick-card {
      padding: 8px;
    }

    .keys-display span {
      padding: 4px 7px;
    }
  }
</style>
