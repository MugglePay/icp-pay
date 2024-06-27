import Result "mo:base/Result";
import Blob "mo:base/Blob";
module {

  public type Subaccount = Blob;

  public type Account = {
    owner : Principal;
    subaccount : ?Subaccount;
  };

  public type ForwardAddress = {
    address : Principal;
    percentage : Float;
  };

  public type ForwardTransaction = {
    tokenName : Text;
    recipient : Principal;
    amount : Nat;
    percentage : Float;
    timestamp : Int;
    isSent : Bool;
    errorMessage : ?Text;
  };
  public type TransferResult = {
    #success : Nat;
    #error : Text;
  };
  public type CanisterData = {
    tokenName : Text;
    ledgerCan : Text;
    latestTransactionIndex : Nat;
    contractBalance:Nat;
    transferFee:Nat;
  };

  public type Result<T, E> = Result.Result<T, E>;
  public type RecurringPayments = {
    amount : Nat;
    lastPaymentDate : Int;
  };

  public type RecurringPaymentHistory = {
    user : Principal;
    amount : Nat;
    receiver : Principal;
    isPaid : Bool;
    timestamp : Int;
  };

};
