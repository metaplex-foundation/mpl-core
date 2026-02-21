import {
  BaseAgentIdentity,
  BaseAgentIdentityInitInfoArgs,
  BaseAgentIdentityUpdateInfoArgs,
  ExternalRegistryRecord,
} from '../generated';
import { LifecycleChecks, lifecycleChecksToBase } from './lifecycleChecks';
import { PluginAuthority, pluginAuthorityToBase } from './pluginAuthority';
import { ExternalPluginAdapterManifest } from './externalPluginAdapterManifest';
import { BaseExternalPluginAdapter } from './externalPluginAdapters';
import { ExternalPluginAdapterKey } from './externalPluginAdapterKey';

export type AgentIdentity = BaseAgentIdentity;

export type AgentIdentityPlugin = BaseExternalPluginAdapter &
  BaseAgentIdentity & {
    type: 'AgentIdentity';
  };

export type AgentIdentityInitInfoArgs = Omit<
  BaseAgentIdentityInitInfoArgs,
  'initPluginAuthority' | 'lifecycleChecks'
> & {
  type: 'AgentIdentity';
  initPluginAuthority?: PluginAuthority;
  lifecycleChecks: LifecycleChecks;
};

export type AgentIdentityUpdateInfoArgs = Omit<
  BaseAgentIdentityUpdateInfoArgs,
  'lifecycleChecks'
> & {
  key: ExternalPluginAdapterKey;
  lifecycleChecks?: LifecycleChecks;
};

export function agentIdentityInitInfoArgsToBase(
  a: AgentIdentityInitInfoArgs
): BaseAgentIdentityInitInfoArgs {
  return {
    uri: a.uri,
    lifecycleChecks: lifecycleChecksToBase(a.lifecycleChecks),
    initPluginAuthority: a.initPluginAuthority
      ? pluginAuthorityToBase(a.initPluginAuthority)
      : null,
  };
}

export function agentIdentityUpdateInfoArgsToBase(
  a: AgentIdentityUpdateInfoArgs
): BaseAgentIdentityUpdateInfoArgs {
  return {
    uri: a.uri ?? null,
    lifecycleChecks: a.lifecycleChecks
      ? lifecycleChecksToBase(a.lifecycleChecks)
      : null,
  };
}

export function agentIdentityFromBase(
  s: BaseAgentIdentity,
  r: ExternalRegistryRecord,
  account: Uint8Array
): AgentIdentity {
  return {
    ...s,
  };
}

export const agentIdentityManifest: ExternalPluginAdapterManifest<
  AgentIdentity,
  BaseAgentIdentity,
  AgentIdentityInitInfoArgs,
  BaseAgentIdentityInitInfoArgs,
  AgentIdentityUpdateInfoArgs,
  BaseAgentIdentityUpdateInfoArgs
> = {
  type: 'AgentIdentity',
  fromBase: agentIdentityFromBase,
  initToBase: agentIdentityInitInfoArgsToBase,
  updateToBase: agentIdentityUpdateInfoArgsToBase,
};
