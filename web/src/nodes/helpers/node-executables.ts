type ExecutableStatus = 'idle' | 'running' | 'interupted' | 'complete' | 'error'

type ExcutableIcon = {
  iconLibrary: 'stencila' | 'default'
  icon: string
}

const execStatusIcons: Partial<Record<ExecutableStatus, ExcutableIcon>> = {
  idle: { iconLibrary: 'default', icon: 'play-circle' },
  running: { iconLibrary: 'default', icon: 'pause-circle' },
}

const executableIcon = (status: ExecutableStatus) => {
  return execStatusIcons[status]
}

export { executableIcon }
export type { ExecutableStatus }
