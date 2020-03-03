import { action, observable } from 'mobx';

import ErrorContainer from './ErrorContainer';
import { CasperService, decodeBase16, decodeBase64, DeployUtil, encodeBase16 } from 'casperlabs-sdk';
import { FieldState, FormState } from 'formstate';
import { isBase16, isBlockHashBase16, isInt, numberBigThan, valueRequired } from '../lib/FormsValidator';
import validator from 'validator';
import * as nacl from 'tweetnacl-ts';
import $ from 'jquery';
import { Deploy } from 'casperlabs-grpc/io/casperlabs/casper/consensus/consensus_pb';
import {
  BigInt as ValueBigInt,
  IntList,
  Key,
  StringList
} from 'casperlabs-grpc/io/casperlabs/casper/consensus/state_pb';
import paymentWASMUrl from '../standard_payment.png';

export enum ContractType {
  WASM = 'WASM',
  Hash = 'Hash'
}

export enum ArgumentType {
  BYTES_VALUE = 'Bytes',
  INT_VALUE = 'Int',
  LONG_VALUE = 'Long',
  BIG_INT = 'BigInt',
  STRING_VALUE = 'String',
  INT_LIST = 'Int List',
  STRING_LIST = 'String List',
  KEY = 'Key',
}

export enum KeyType {
  ADDRESS = 'Address',
  HASH = 'Hash',
  UREF = 'URef',
  LOCAL = 'Local'
}

export enum BitWidth {
  B_128 = 128,
  B_256 = 256,
  B_512 = 512
}

export type DeployArgument = {
  name: FieldState<string>,
  type: FieldState<ArgumentType>,
  // if type == ArgumentType.Key then the type of secondType is KeyType
  // and if type == ArgumentType.BIG_INT, then the type of secondType is BitWidth
  // otherwise second equals to null
  secondType: FieldState<KeyType | BitWidth | null>,
  URefAccessRight: FieldState<Key.URef.AccessRightsMap[keyof Key.URef.AccessRightsMap]>, // null if type != ArgumentType.KEY
  value: FieldState<string>
}

export type FormDeployArgument = FormState<DeployArgument>;
type FormDeployArguments = FormState<FormDeployArgument[]>;

export type DeployConfiguration = {
  contractType: FieldState<ContractType | null>,
  contractHash: FieldState<string>,
  gasPrice: FieldState<number>,
  gasLimit: FieldState<number>,
  fromAddress: FieldState<string>
}

export type FormDeployConfiguration = FormState<DeployConfiguration>;

export class DeployContractsContainer {
  @observable deployConfiguration: FormDeployConfiguration = new FormState<DeployConfiguration>({
    contractType: new FieldState<ContractType | null>(null).validators(valueRequired),
    contractHash: new FieldState('').disableAutoValidation().validators(isBlockHashBase16),
    gasPrice: new FieldState<number>(10).validators(
      numberBigThan(0),
      isInt
    ),
    gasLimit: new FieldState<number>(10000000).validators(
      numberBigThan(0),
      isInt
    ),
    fromAddress: new FieldState<string>('')
  });
  @observable deployArguments: FormDeployArguments = new FormState<FormDeployArgument[]>([]);
  @observable editingDeployArguments: FormDeployArguments = new FormState<FormDeployArgument[]>([]);
  @observable privateKey = new FieldState<string>('');
  @observable selectedFile: File | null = null;
  @observable editing: boolean = false;
  @observable signDeployModal: boolean = false;
  private selectedFileContent: null | ByteArray = null;

  // id for accordion
  accordionId = 'deploy-table-accordion';

  constructor(
    private errors: ErrorContainer,
    private casperService: CasperService
  ) {
  }

  @action.bound
  removeDeployArgument(deployArgument: FormDeployArgument) {
    let i = this.deployArguments.$.findIndex((f) => f === deployArgument);
    this.deployArguments.$.splice(i, 1);
  }

  @action.bound
  addNewEditingDeployArgument() {
    this.editing = true;
    let newDeployArgument = new FormState({
      name: new FieldState<string>('').disableAutoValidation().validators(valueRequired),
      type: new FieldState<ArgumentType>(ArgumentType.STRING_VALUE),
      secondType: new FieldState<KeyType | BitWidth | null>(null),
      URefAccessRight: new FieldState<Key.URef.AccessRightsMap[keyof Key.URef.AccessRightsMap]>(Key.URef.AccessRights.UNKNOWN),
      value: new FieldState<string>('').disableAutoValidation().validators(valueRequired)
    }).compose().validators(this.validateDeployArgument);


    this.editingDeployArguments.$.push(newDeployArgument);
  }

  @action.bound
  handleFileSelect(e: React.ChangeEvent<HTMLInputElement>) {
    if (e.target.files) {
      this.selectedFileContent = null;
      this.selectedFile = e.target.files[0];
      const reader = new FileReader();
      reader.readAsArrayBuffer(this.selectedFile);
      reader.onload = e => {
        this.selectedFileContent = new Uint8Array(reader.result as ArrayBuffer);
      };
    }
  }

  @action.bound
  async saveEditingDeployArguments() {
    const res = await this.editingDeployArguments.validate();
    if (!res.hasError) {
      while (this.editingDeployArguments.$.length) {
        this.deployArguments.$.push(this.editingDeployArguments.$.shift()!);
      }
      this.editing = false;
    }
  }

  @action.bound
  cancelEditing() {
    this.editingDeployArguments.$.splice(0, this.editingDeployArguments.$.length);
    this.editing = false;
  }

  @action.bound
  clearForm() {
    let msg = 'Do you want to clear the form?';
    if (window.confirm(msg)) {
      this.deployConfiguration.reset();
      this.editing = false;
      this.editingDeployArguments.reset();
      this.deployArguments.reset();
    }
  }


  @action.bound
  async openSignModal() {
    let v1 = await this.deployConfiguration.validate();
    let v2 = await this.deployArguments.validate();
    if (v1.hasError || v2.hasError) {
      return;
    } else {
      this.signDeployModal = true;
    }
  }

  @action.bound
  async onSubmit() {
    let deploy = await this.makeDeploy();
    let keyPair = nacl.sign_keyPair_fromSecretKey(decodeBase64(this.privateKey.$));
    let signedDeploy = DeployUtil.signDeploy(deploy!, keyPair);
    try {
      await this.errors.withCapture(this.casperService.deploy(signedDeploy));
      ($(`#${this.accordionId}`) as any).collapse('hide');
      console.log(encodeBase16(signedDeploy.getDeployHash_asU8()));
      return true;
    } catch {
      return true;
    }
  }

  private async makeDeploy(): Promise<Deploy | null> {
    let deployConfigurationForm = await this.deployConfiguration.validate();
    let deployArguments = await this.deployArguments.validate();
    if (deployConfigurationForm.hasError || deployArguments.hasError) {
      return null;
    } else {
      const config = deployConfigurationForm.value;
      const args = deployArguments.value;
      let type: 'Hash' | 'WASM';
      let session: ByteArray;
      if (config.contractType.value === ContractType.Hash) {
        type = 'Hash';
        session = decodeBase16(config.contractHash.value);
      } else {
        type = 'WASM';
        session = this.selectedFileContent!;
      }
      const gasLimit = config.gasLimit.value;
      const gasPrice = config.gasPrice.value;
      let argsProto = args.map((arg: FormState<DeployArgument>) => {
        const deployArg = new Deploy.Arg();
        const value = new Deploy.Arg.Value();
        deployArg.setName(arg.$.name.value);
        const argValueStr: string = arg.$.value.value;
        switch (arg.$.type.value) {
          case ArgumentType.BYTES_VALUE:
            value.setBytesValue(decodeBase16(argValueStr));
            break;
          case ArgumentType.INT_VALUE:
            value.setIntValue(parseInt(argValueStr));
            break;
          case ArgumentType.LONG_VALUE:
            value.setLongValue(parseInt(argValueStr));
            break;
          case ArgumentType.BIG_INT:
            const bigInt = new ValueBigInt();
            bigInt.setValue(argValueStr);
            bigInt.setBitWidth(arg.$.secondType as unknown as BitWidth);
            break;
          case ArgumentType.STRING_VALUE:
            value.setStringValue(argValueStr);
            break;
          case ArgumentType.INT_LIST:
            const intList = new IntList();
            let intListValue = JSON.parse(argValueStr) as Array<number>;
            intList.setValuesList(intListValue);
            value.setIntList(intList);
            break;
          case ArgumentType.STRING_LIST:
            const stringList = new StringList();
            const stringListValue = JSON.parse(argValueStr) as Array<string>;
            stringList.setValuesList(stringListValue);
            value.setStringList(stringList);
            break;
          case ArgumentType.KEY:
            const key = new Key();
            let keyType = arg.$.secondType.value as KeyType;
            let valueInByteArray = decodeBase16(argValueStr);
            switch (keyType) {
              case KeyType.ADDRESS:
                const address = new Key.Address();
                address.setAccount(valueInByteArray);
                key.setAddress(address);
                break;
              case KeyType.HASH:
                const hash = new Key.Hash();
                hash.setHash(valueInByteArray);
                key.setHash(hash);
                break;
              case KeyType.UREF:
                const URef = new Key.URef();
                URef.setUref(valueInByteArray);
                URef.setAccessRights(arg.$.URefAccessRight.value!);
                break;
              case KeyType.LOCAL:
                const local = new Key.Local();
                local.setHash(valueInByteArray);
                break;
            }
            value.setKey(key);
            break;
        }
        deployArg.setValue(value);
        return deployArg;
      });
      let wasmRequest = await fetch(paymentWASMUrl);
      let paymentWASM: ArrayBuffer = await wasmRequest.arrayBuffer();
      return DeployUtil.makeDeploy(argsProto, type, session, new Uint8Array(paymentWASM), BigInt(gasLimit), new Uint8Array(0), gasPrice);
    }
  }


  private validateDeployArgument(deployArgument: DeployArgument): string | false {
    let isArrayOf = (str: string, type: 'int' | 'string'): false | string => {
      let obj;
      let errMsg = `Value is not an array of ${type}.`;
      try {
        obj = JSON.parse(str);
      } catch {
        return errMsg;
      }
      if (!Array.isArray(obj)) {
        return errMsg;
      } else {
        let res = (obj as Array<any>).every(v => {
          if (type === 'string') {
            return typeof v === 'string';
          } else {
            return Number.isInteger(v);
          }
        });
        return !res && errMsg;
      }
    };

    const value = deployArgument.value.$;
    switch (deployArgument.type.$) {
      case ArgumentType.INT_VALUE:
        if (!validator.isInt(value, { min: -2147483648, max: 2147483647 })) {
          return 'Value should be an Integer';
        }
        return false;
      case ArgumentType.BYTES_VALUE:
        return isBase16(value);
      case ArgumentType.INT_LIST:
        return isArrayOf(value, 'int');
      case ArgumentType.STRING_VALUE:
        return false;
      case ArgumentType.STRING_LIST:
        return isArrayOf(value, 'string');
      case ArgumentType.BIG_INT:
        break;
      case ArgumentType.KEY:
        break;
      case ArgumentType.LONG_VALUE:
        if (!validator.isInt(value)) {
          return 'Value should be an Long';
        }
        return false;
    }
    return false;
  }
}
