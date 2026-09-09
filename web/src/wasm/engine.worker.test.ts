// @vitest-environment node
import { afterEach, expect, it, vi } from 'vitest'
import type { EngineRequest, EngineResponse } from './engine.worker'

vi.mock('./pkg/frees.js', () => ({
  default: async () => ({}),
  solve_zerocopy: () => ({ envelope: '{}', odeBuffers: [new Float64Array([0, 1])] }),
  solve_table_zerocopy: () => ({ envelope: '{}', matrix: new Float64Array([2, 4]) }),
}))

afterEach(() => {
  vi.unstubAllGlobals()
  delete (globalThis as { __freesOnProgress?: unknown }).__freesOnProgress
  vi.resetModules()
})

it('transfers sweep and trajectory buffers instead of cloning them', async () => {
  const received: EngineResponse[] = []
  const sent: Float64Array[] = []
  const worker = {
    onmessage: null as ((event: { data: EngineRequest }) => void) | null,
    postMessage(message: EngineResponse, transfer: ArrayBuffer[] = []) {
      if ('ok' in message && message.ok) {
        sent.push(...(message.odeBuffers ?? []), ...(message.matrix ? [message.matrix] : []))
      }
      received.push(structuredClone(message, { transfer }))
    },
  }
  vi.stubGlobal('self', worker)
  await import('./engine.worker')
  for (const [id, method] of (['solve', 'solveTable'] as const).entries()) {
    worker.onmessage!({ data: { id, method, args: ['', '{}'] } })
    await vi.waitFor(() => expect(received).toHaveLength(id + 1))
  }
  expect(sent.map((array) => array.byteLength)).toEqual([0, 0])
  expect(received).toEqual([
    { id: 0, ok: true, result: '{}', odeBuffers: [new Float64Array([0, 1])] },
    { id: 1, ok: true, result: '{}', matrix: new Float64Array([2, 4]) },
  ])
})
