import { describe, expect, it, vi } from 'vitest';
import { readFileSync } from 'node:fs';
import { resolve } from 'node:path';
import CmmnModdle from 'cmmn-moddle/lib/simple.js';
import ssw from '../src/ssw-cmmn-moddle.json';
import { cmmnDefinitionFor, cmmnLabelFor, cmmnMetadataFor, updateCmmnProperties } from '../src/diagram-adapters.ts';

const root = resolve(import.meta.dirname, '../..');
const readFixture = (name) => readFileSync(resolve(root, `fixtures/cmmn/${name}.cmmn`), 'utf8');

function parse(moddle, xml) {
  return new Promise((resolvePromise, reject) => moddle.fromXML(xml, (error, definitions, context) => error ? reject(error) : resolvePromise({ definitions, warnings: context.warnings || [] })));
}

function serialize(moddle, definitions) {
  return new Promise((resolvePromise, reject) => moddle.toXML(definitions, { format: true }, (error, xml) => error ? reject(error) : resolvePromise(xml)));
}

describe('CMMN authored identity', () => {
  it('round-trips package, member, Process Task, IDs, and CMMNDI through cmmn-moddle', async () => {
    const moddle = new CmmnModdle({ ssw });
    const { definitions, warnings } = await parse(moddle, readFixture('roundtrip'));
    expect(warnings).toEqual([]);
    expect(definitions.packageName).toBe('cybling.sdk');
    const casePlan = definitions.cases[0].casePlanModel;
    const planItem = casePlan.planItems[0];
    expect(planItem.id).toBe('PlanItem_Birth');
    expect(planItem.architecturalName).toBe('cybling.sdk.Birth');
    expect(planItem.definitionRef.$type).toBe('cmmn:ProcessTask');
    const xml = await serialize(moddle, definitions);
    expect(xml).toContain('ssw:packageName="cybling.sdk"');
    expect(xml).toContain('ssw:architecturalName="cybling.sdk.Birth"');
    expect(xml).toContain('cmmnElementRef="PlanItem_Birth"');
  });

  it('parses the representative Stage, Case Task, Process Task, Sentry, and criterion fixture', async () => {
    const moddle = new CmmnModdle({ ssw });
    const { definitions } = await parse(moddle, readFixture('valid'));
    const casePlan = definitions.cases[0].casePlanModel;
    expect(casePlan.planItemDefinitions.map((item) => item.$type)).toEqual(expect.arrayContaining(['cmmn:Stage', 'cmmn:ProcessTask', 'cmmn:CaseTask']));
    expect(casePlan.sentries[0].onParts[0].sourceRef.id).toBe('PlanItem_Sdk');
    expect(casePlan.planItems.find((item) => item.id === 'PlanItem_Birth').entryCriteria[0].sentryRef.id).toBe('Sentry_SdkAvailable');
  });

  it('unwraps CMMN connection metadata and plan-item definitions behind the adapter boundary', () => {
    const association = { $type: 'cmmn:Association', id: 'Association_1', name: 'Need trace', architecturalName: 'cybling#needTrace' };
    const edge = { businessObject: { $type: 'cmmndi:CMMNEdge', id: 'Association_1_di', cmmnElementRef: association } };
    const processTask = { $type: 'cmmn:ProcessTask', id: 'ProcessTask_Birth' };
    const planItem = { businessObject: { $type: 'cmmn:PlanItem', id: 'PlanItem_Birth', definitionRef: processTask } };

    expect(cmmnMetadataFor(edge)).toBe(association);
    expect(cmmnDefinitionFor(edge)).toBe(association);
    expect(cmmnDefinitionFor(planItem)).toBe(processTask);
  });

  it('reads and updates the rendered plan-item Label while preserving semantic edge updates', () => {
    const processTask = { $type: 'cmmn:ProcessTask', id: 'ProcessTask_Birth', name: 'Definition fallback' };
    const shape = { id: 'PlanItem_Birth', businessObject: { $type: 'cmmn:PlanItem', id: 'PlanItem_Birth', name: 'Provision a Cybling', definitionRef: processTask } };
    const association = { $type: 'cmmn:Association', id: 'Association_1', name: 'Need trace' };
    const edge = { id: 'Association_1_di', businessObject: { $type: 'cmmndi:CMMNEdge', cmmnElementRef: association } };
    const updateProperties = vi.fn();
    const modeler = { get: vi.fn(() => ({ updateProperties })) };

    expect(cmmnLabelFor(shape)).toBe('Provision a Cybling');
    delete shape.businessObject.name;
    expect(cmmnLabelFor(shape)).toBe('Definition fallback');
    updateCmmnProperties(modeler, shape, { name: 'Birth onboarding' });
    expect(updateProperties).toHaveBeenCalledWith(shape, { name: 'Birth onboarding' });
    updateCmmnProperties(modeler, edge, { name: 'Provisioning trace' });
    expect(updateProperties).toHaveBeenCalledWith(association, { name: 'Provisioning trace' }, edge);
  });
});
