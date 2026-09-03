import { describe, expect, it } from 'vitest';
import { BpmnModdle } from 'bpmn-moddle';
import CmmnModdle from 'cmmn-moddle/lib/simple.js';
import bpmnSsw from '../src/ssw-moddle.json';
import cmmnSsw from '../src/ssw-cmmn-moddle.json';
import { implementationStatus, NODE_STATUSES } from '../src/core.js';

const parse = (moddle, xml) => new Promise((resolve, reject) => moddle.fromXML(xml, (error, definitions) => error ? reject(error) : resolve(definitions)));
const serialize = (moddle, definitions) => new Promise((resolve, reject) => moddle.toXML(definitions, { format: true }, (error, xml) => error ? reject(error) : resolve(xml)));

describe('Implementation Status XML', () => {
  for (const status of ['new', 'modify', 'locked']) {
    it(`round-trips BPMN ${status}`, async () => {
      const moddle = new BpmnModdle({ ssw: bpmnSsw });
      const { rootElement: definitions } = await moddle.fromXML(`<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" xmlns:ssw="https://software-schematic.dev/schema/bpmn"><bpmn:process id="Process_1"><bpmn:task id="Task_1" ssw:implementationStatus="${status}"/></bpmn:process></bpmn:definitions>`);
      const task = definitions.rootElements[0].flowElements[0];
      expect(implementationStatus(task)).toBe(status);
      expect((await moddle.toXML(definitions, { format: true })).xml).toContain(`ssw:implementationStatus="${status}"`);
      expect(NODE_STATUSES[status].color).toMatch(/^#/);
      expect(NODE_STATUSES[status].meaning).toBeTruthy();
    });
  }

  it('round-trips CMMN status and removes default open metadata', async () => {
    const moddle = new CmmnModdle({ ssw: cmmnSsw });
    const definitions = await parse(moddle, '<cmmn:definitions xmlns:cmmn="http://www.omg.org/spec/CMMN/20151109/MODEL" xmlns:ssw="https://software-schematic.dev/schema/cmmn"><cmmn:case id="Case_1"><cmmn:casePlanModel id="Plan_1"><cmmn:task id="TaskDef_1"/><cmmn:planItem id="Item_1" definitionRef="TaskDef_1" ssw:implementationStatus="modify"/></cmmn:casePlanModel></cmmn:case></cmmn:definitions>');
    const item = definitions.cases[0].casePlanModel.planItems[0];
    expect(implementationStatus(item)).toBe('modify');
    item.implementationStatus = undefined;
    const xml = await serialize(moddle, definitions);
    expect(xml).not.toContain('implementationStatus');
    expect(implementationStatus(item)).toBe('open');
    expect(implementationStatus({ implementationStatus: 'invalid' })).toBe('open');
  });

  it('persists newly assigned statuses across a serialize and reload cycle', async () => {
    const bpmn = new BpmnModdle({ ssw: bpmnSsw });
    const { rootElement: bpmnDefinitions } = await bpmn.fromXML('<bpmn:definitions xmlns:bpmn="http://www.omg.org/spec/BPMN/20100524/MODEL" xmlns:ssw="https://software-schematic.dev/schema/bpmn"><bpmn:process id="Process_1"><bpmn:task id="Task_1"/></bpmn:process></bpmn:definitions>');
    bpmnDefinitions.rootElements[0].flowElements[0].implementationStatus = 'new';
    const bpmnReloaded = await bpmn.fromXML((await bpmn.toXML(bpmnDefinitions, { format: true })).xml);
    expect(implementationStatus(bpmnReloaded.rootElement.rootElements[0].flowElements[0])).toBe('new');

    const cmmn = new CmmnModdle({ ssw: cmmnSsw });
    const cmmnDefinitions = await parse(cmmn, '<cmmn:definitions xmlns:cmmn="http://www.omg.org/spec/CMMN/20151109/MODEL" xmlns:ssw="https://software-schematic.dev/schema/cmmn"><cmmn:case id="Case_1"><cmmn:casePlanModel id="Plan_1"><cmmn:task id="TaskDef_1"/><cmmn:planItem id="Item_1" definitionRef="TaskDef_1"/></cmmn:casePlanModel></cmmn:case></cmmn:definitions>');
    cmmnDefinitions.cases[0].casePlanModel.planItems[0].implementationStatus = 'new';
    const cmmnReloaded = await parse(cmmn, await serialize(cmmn, cmmnDefinitions));
    expect(implementationStatus(cmmnReloaded.cases[0].casePlanModel.planItems[0])).toBe('new');
  });
});
