<script lang="ts">
  import { onDestroy, onMount, tick } from 'svelte'
  import { invoke } from '@tauri-apps/api/core'
  import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
  import { writeText } from '@tauri-apps/plugin-clipboard-manager'
  import {
    containers,
    containersError,
    containersLoading,
    type ContainerInfo,
  } from './lib/containers'

  let busyIds = new Set<string>()
  let copiedId = ''
  let maxContainerCount = 0
  let hasInitialResize = false
  const refreshIntervalMs = 10000

  const windowHandle = getCurrentWindow()

  const readCssPx = (name: string, fallback: number) => {
    if (typeof window === 'undefined') {
      return fallback
    }
    const value = getComputedStyle(document.documentElement).getPropertyValue(name)
    const parsed = Number.parseFloat(value)
    return Number.isFinite(parsed) ? parsed : fallback
  }

  const computeWindowHeight = (count: number) => {
    const rowHeight = readCssPx('--row-height', 56)
    const rowGap = readCssPx('--row-gap', 8)
    const appPadding = readCssPx('--app-padding', 16)
    const controlsHeight = readCssPx('--controls-height', 44)
    const emptyHeight = readCssPx('--empty-height', 64)
    const extra = readCssPx('--window-extra', 24)
    const listHeight =
      count > 0
        ? count * rowHeight + Math.max(0, count - 1) * rowGap
        : emptyHeight
    return appPadding * 2 + controlsHeight + listHeight + extra
  }

  const computeWindowWidth = () => readCssPx('--window-width', 760)

  const measureWindowHeight = () => {
    if (typeof window === 'undefined') {
      return null
    }
    const app = document.querySelector('.app')
    if (!app) {
      return null
    }
    const extra = readCssPx('--window-extra', 24)
    return app.getBoundingClientRect().height + extra
  }

  const applyWindowSize = async (count: number) => {
    if (typeof window === 'undefined') {
      return
    }
    await tick()
    const measured = measureWindowHeight()
    const targetHeight = Math.ceil(measured ?? computeWindowHeight(count))
    const targetWidth = Math.ceil(computeWindowWidth())
    await windowHandle.setSize(new LogicalSize(targetWidth, targetHeight))
  }

  const updateWindowSize = async (count: number) => {
    if (!hasInitialResize) {
      maxContainerCount = count
      hasInitialResize = true
      await applyWindowSize(count)
      return
    }

    if (count > maxContainerCount) {
      maxContainerCount = count
      await applyWindowSize(count)
    }
  }

  const refresh = async () => {
    containersLoading.set(true)
    containersError.set(null)
    try {
      const data = (await invoke('list_containers')) as ContainerInfo[]
      containers.set(data)
      await updateWindowSize(data.length)
    } catch (err) {
      containersError.set(normalizeError(err))
    } finally {
      containersLoading.set(false)
    }
  }

  const runAction = async (action: string, containerId: string) => {
    setBusy(containerId, true)
    containersError.set(null)
    try {
      await invoke(action, { containerId })
      await refresh()
    } catch (err) {
      containersError.set(normalizeActionError(action, err))
    } finally {
      setBusy(containerId, false)
    }
  }

  const copyExec = async (containerId: string) => {
    containersError.set(null)
    try {
      await writeText(`docker exec -it ${containerId} /bin/bash`)
      copiedId = containerId
      window.setTimeout(() => {
        if (copiedId === containerId) {
          copiedId = ''
        }
      }, 2000)
    } catch (err) {
      containersError.set(normalizeError(err))
    }
  }

  const setBusy = (containerId: string, isBusy: boolean) => {
    const next = new Set(busyIds)
    if (isBusy) {
      next.add(containerId)
    } else {
      next.delete(containerId)
    }
    busyIds = next
  }

  const normalizeActionError = (action: string, err: unknown) => {
    const base = normalizeError(err)
    if (action === 'start_container') {
      return `Failed to start container. ${base}`
    }
    if (action === 'stop_container') {
      return `Failed to stop container. ${base}`
    }
    if (action === 'restart_container') {
      return `Failed to restart container. ${base}`
    }
    return base
  }

  const normalizeError = (err: unknown) => {
    if (err instanceof Error && err.message) {
      return err.message
    }
    if (typeof err === 'string') {
      return err
    }
    return 'Unexpected error occurred.'
  }

  const formatName = (container: ContainerInfo) => {
    if (container.name) {
      return container.name
    }
    return container.id.slice(0, 12)
  }

  const formatStateLine = (container: ContainerInfo) => {
    const status = container.status ? `Status: ${container.status}` : 'Status: -'
    return status
  }

  const getStateClass = (state: string) => {
    const normalized = state.trim().toLowerCase()
    if (normalized === 'running') {
      return 'state running'
    }
    if (normalized === 'exited' || normalized === 'stopped') {
      return 'state stopped'
    }
    if (normalized === 'paused') {
      return 'state paused'
    }
    return 'state unknown'
  }

  const formatStateLabel = (state: string) => {
    return state ? state : 'unknown'
  }

  const closeWindow = async () => {
    await windowHandle.close()
  }

  const startAutoRefresh = () => {
    if (typeof window === 'undefined') {
      return null
    }
    const intervalId = window.setInterval(() => {
      void refresh()
    }, refreshIntervalMs)
    return () => window.clearInterval(intervalId)
  }

  const stopAutoRefresh = startAutoRefresh()

  onMount(refresh)
  onDestroy(() => {
    if (stopAutoRefresh) {
      stopAutoRefresh()
    }
  })
</script>

<main class="app">
  <div class="controls">
    <button class="button ghost" on:click={refresh}>Refresh</button>
    <button class="button danger" on:click={closeWindow}>Close</button>
  </div>

  {#if $containersError}
    <section class="alert">{$containersError}</section>
  {/if}

  <section class="list">
    {#if $containers.length === 0 && !$containersLoading}
      <div class="empty">No containers found.</div>
    {/if}

    {#each $containers as container (container.id)}
      <article class="row">
        <div class="container-info">
          <div class="name">{formatName(container)}</div>
          <div class="meta">
            <span class={getStateClass(container.state)}>
              {formatStateLabel(container.state)}
            </span>
            <span class="status">{formatStateLine(container)}</span>
          </div>
        </div>
        <div class="actions">
          <button
            class="button"
            on:click={() => runAction('start_container', container.id)}
            disabled={busyIds.has(container.id)}
          >
            Start
          </button>
          <button
            class="button"
            on:click={() => runAction('stop_container', container.id)}
            disabled={busyIds.has(container.id)}
          >
            Stop
          </button>
          <button
            class="button"
            on:click={() => runAction('restart_container', container.id)}
            disabled={busyIds.has(container.id)}
          >
            Restart
          </button>
          <button
            class="button ghost"
            on:click={() => copyExec(container.id)}
            disabled={busyIds.has(container.id)}
          >
            {copiedId === container.id ? 'Copied' : 'Copy exec'}
          </button>
        </div>
      </article>
    {/each}
  </section>
</main>
