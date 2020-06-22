import * as CL from "../../../../contract-as/assembly";
import {Error, ErrorCode} from "../../../../contract-as/assembly/error";
import {fromBytesString, fromBytesI32} from "../../../../contract-as/assembly/bytesrepr";
import {arrayToTyped} from "../../../../contract-as/assembly/utils";
import {Key, PublicKey, PUBLIC_KEY_ED25519_ID} from "../../../../contract-as/assembly/key"
import {addAssociatedKey, AddKeyFailure, ActionType, setActionThreshold, SetThresholdFailure} from "../../../../contract-as/assembly/account";

const ARG_KEY_MANAGEMENT_THRESHOLD = "key_management_threshold";
const ARG_DEPLOY_THRESHOLD = "deploy_threshold";

export function call(): void {
  let publicKeyBytes = new Array<u8>(32);
  publicKeyBytes.fill(123);
  let publicKey = new PublicKey(PUBLIC_KEY_ED25519_ID, arrayToTyped(publicKeyBytes));

  const addResult = addAssociatedKey(publicKey, 100);
  switch (addResult) {
    case AddKeyFailure.DuplicateKey:
      break;
    case AddKeyFailure.Ok:
      break;
    default:
      Error.fromUserError(50).revert();
      break;
  }

  let keyManagementThresholdBytes = CL.getNamedArg(ARG_KEY_MANAGEMENT_THRESHOLD);
  let keyManagementThreshold = keyManagementThresholdBytes[0];

  let deployThresholdBytes = CL.getNamedArg(ARG_DEPLOY_THRESHOLD);
  let deployThreshold = deployThresholdBytes[0];

  if (keyManagementThreshold != 0) {
    if (setActionThreshold(ActionType.KeyManagement, keyManagementThreshold) != SetThresholdFailure.Ok) {
      // TODO: Create standard Error from those enum values
      Error.fromUserError(4464 + 1).revert();
    }
  }
  if (deployThreshold != 0) {
    if (setActionThreshold(ActionType.Deployment, deployThreshold) != SetThresholdFailure.Ok) {
      Error.fromUserError(4464).revert();
      return;
    }
  }
  
}
