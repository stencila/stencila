import { InstructionType, NodeType } from '@stencila/types'

/**
 * Specification of a kernel
 *
 * See `stencila kernels list --as json` for available fields.
 */
export interface Kernel {
  name: string
  type: string
  languages: string[]
}

/**
 * Specification of a prompt
 *
 * See `stencila prompts list --as json` for available fields.
 */
export interface Prompt {
  id: string
  name: string
  version: string
  description: string
  instructionTypes: InstructionType[]
  instructionPatterns: string[]
  nodeTypes: NodeType[]
}

/**
 * Specification of a model
 *
 * See `stencila models list --as json` for available fields.
 */
export interface Model {
  id: string
  provider: string
  name: string
  version: string
}

/**
 * A bus for Stencila system data
 *
 * Clients update these data when it changes on the system.
 * Components can listen for relevant updates and get the data
 * as needed.
 */
class Data extends EventTarget {
  private _kernels: Kernel[]
  private _prompts: Prompt[]
  private _models: Model[]

  constructor() {
    super()
    this._kernels = []
    this._prompts = []
    this._models = []
  }

  private emit(eventName: string) {
    const event = new CustomEvent(eventName)
    this.dispatchEvent(event)
  }

  get kernels(): Kernel[] {
    return this._kernels
  }

  set kernels(kernels: Kernel[]) {
    if (this._kernels !== kernels) {
      this._kernels = kernels
      this.emit('kernels')
    }
  }

  get prompts(): Prompt[] {
    return this._prompts
  }

  set prompts(prompts: Prompt[]) {
    if (this._prompts !== prompts) {
      this._prompts = prompts
      this.emit('prompts')
    }
  }

  get models(): Model[] {
    return this._models
  }

  set models(models: Model[]) {
    if (this._models !== models) {
      this._models = models
      this.emit('models')
    }
  }
}

export const data = new Data()
