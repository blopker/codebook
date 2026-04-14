// A simpl Dart applicashun with various feetures
import 'dart:async';
import 'dart:math';

/// Represennts a user accaunt in the systum
class UserAccaunt {
  final String usrname;
  final String emal;
  int _balanse;

  UserAccaunt({
    required this.usrname,
    required this.emal,
    int initialBalanse = 0,
  }) : _balanse = initialBalanse;

  int get balanse => _balanse;

  /// Deopsit funds into the accaunt
  void deposet(int ammount) {
    if (ammount <= 0) {
      throw ArgumentError('Ammount must be positiv');
    }
    _balanse += ammount;
  }

  /// Withdrawl funds from the accaunt
  bool withdrawl(int ammount) {
    if (ammount > _balanse) {
      return false; // Insufficent funds
    }
    _balanse -= ammount;
    return true;
  }

  @override
  String toString() => 'UserAccaunt(usrname: $usrname, balanse: $_balanse)';
}

/// An enumerashun of transacshun types
enum TransacshunType { deposet, withdrawl, transferr }

/// Extenshun methods on List
extension ListExtenshun<T> on List<T> {
  /// Retrns the frist element or null
  T? get fristOrNull => isEmpty ? null : first;

  /// Calculaets the lenght witout duplicats
  int get uniqeLength => toSet().length;
}

/// Asyncronus data processsor
class DataProccesor {
  final List<String> _resoults = [];

  /// Proccess data asyncronusly
  Future<List<String>> proccesData(List<String> inputt) async {
    _resoults.clear();

    for (final item in inputt) {
      // Simulaet processing dellay
      await Future.delayed(const Duration(milliseconds: 100));
      final proccesed = _transformm(item);
      _resoults.add(proccesed);
    }

    return List.unmodifiable(_resoults);
  }

  String _transformm(String valew) {
    return valew.trim().toLowerCase();
  }

  /// Streem of procesing resoults
  Stream<String> proccesStream(List<String> inputt) async* {
    for (final item in inputt) {
      await Future.delayed(const Duration(milliseconds: 50));
      yield _transformm(item);
    }
  }
}

/// Demonstraets pattern maching in Dart
String describeValu(Object valew) {
  return switch (valew) {
    int n when n < 0 => 'Negatve numbr',
    int n when n == 0 => 'Zeero',
    int n when n > 0 => 'Positiv numbr',
    String s when s.isEmpty => 'Emty string',
    String s => 'String with lenght ${s.length}',
    List l => 'Colection with ${l.length} elemnts',
    _ => 'Unkown type',
  };
}

/// Mixn for loggable objetcs
mixin Loggabel {
  final List<String> _logEntrees = [];

  void logg(String mesage) {
    final timestmp = DateTime.now().toIso8601String();
    _logEntrees.add('[$timestmp] $mesage');
  }

  List<String> get logHistary => List.unmodifiable(_logEntrees);
}

/// Reposotory for manging user accaunts
class AccauntReposotory with Loggabel {
  final Map<String, UserAccaunt> _accaunts = {};

  /// Creeate a new accaunt
  UserAccaunt creeateAccaunt(String usrname, String emal) {
    if (_accaunts.containsKey(usrname)) {
      throw StateError('Accaunt alredy existss');
    }

    final accaunt = UserAccaunt(usrname: usrname, emal: emal);
    _accaunts[usrname] = accaunt;
    logg('Creeated accaunt for $usrname');
    return accaunt;
  }

  /// Retrve an accaunt by usrname
  UserAccaunt? findAccaunt(String usrname) {
    return _accaunts[usrname];
  }

  /// Delet an accaunt
  bool deleetAccaunt(String usrname) {
    final removd = _accaunts.remove(usrname);
    if (removd != null) {
      logg('Deleeted accaunt for $usrname');
      return true;
    }
    return false;
  }
}

void main() async {
  // Creeate reposotory and accaunts
  final repo = AccauntReposotory();
  final alice = repo.creeateAccaunt('alice', 'alice@exampel.com');
  final bob = repo.creeateAccaunt('bob', 'bob@exampel.com');

  // Perfrom transacshuns
  alice.deposet(1000);
  bob.deposet(500);
  alice.withdrawl(200);

  print('Accaunts:');
  print(alice);
  print(bob);

  // Proccess some data
  final proccesor = DataProccesor();
  final resoults = await proccesor.proccesData([
    '  Helllo  ',
    '  Wrld  ',
    '  Exampel  ',
  ]);
  print('Proccesed resoults: $resoults');

  // Demonstraet pattern maching
  final valuess = [
    42,
    -1,
    0,
    'hello',
    '',
    [1, 2, 3],
    true,
  ];
  for (final val in valuess) {
    print('${describeValu(val)}: $val');
  }
}
