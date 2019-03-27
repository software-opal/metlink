var ApiEndpoint = 'https://www.metlink.org.nz/api/v1';
var ApiEndpointRouteMap = 'https://www.metlink.org.nz/timetables';

function allServiceUpdates() {
  console.log('ServiceUpdates.all');
  var deferred = $q.defer();

  $http.get(ApiEndpoint + '/ServiceNotices/')
    .success(function(data, status) {

      // Merge Disruptions and Delays into one array
      if (data.Disruptions && data.Disruptions.length > 0) {
        for (var i = 0; i < data.Disruptions.length; i++) {
          if (data.Disruptions[i].AffectedLines) {
            data.Disruptions[i].AffectedLinesArray = data.Disruptions[i].AffectedLines.split(',');
          }

          data.Disruptions[i].Blurb = data.Disruptions[i].Content_Plaintext.substring(0, 100) + '...';

          updates.push(data.Disruptions[i]);
        }
      }

      if (data.Delays && data.Delays.length > 0) {
        for (var i = 0; i < data.Delays.length; i++) {
          if (data.Delays[i].AffectedLines) {
            data.Delays[i].AffectedLinesArray = data.Delays[i].AffectedLines.split(',');
          }

          // Note, Delays have no blurb
          updates.push(data.Delays[i]);
        }
      }

      deferred.resolve(updates);
    })
    .error(function(data, status) {

      deferred.reject(data);
    });

  return deferred.promise;
}

function getAllStops() {
  console.log('getting stops');
  var q = $q.defer();

  $ionicPlatform.ready(function() {
    // First see if we need to clear out our data
    console.log('preferences start');
    Preferences.get('StopDataLastUpdated')
      .then(function(val) {
        console.log(val);
        updateStopData(q, val);
      }, function(error) {
        console.log('no lastupdated found');
        updateStopData(q, false);
      });
  });
  console.log('preferences end');

  return q.promise;
}

function getStopDepartures(sms) {
  console.log('getting departures');

  var deferred = $q.defer();

  $http.get(ApiEndpoint + '/StopDepartures/' + sms)
    .success(function(data, status) {

      // To-Do:
      // Turn favourite services into their own factory?

      // Now get the favourite services for below
      $ionicPlatform.ready(function() {
        db.transaction(function(tx) {
          //tx.executeSql('DELETE FROM Services');
          tx.executeSql('SELECT Code FROM FavouriteServices', [], function(tx, rs) {
            var favServices = [];
            if (rs.rows.length > 0) {
              for (var i = 0; i < rs.rows.length; i++) {
                favServices.push(rs.rows.item(i)
                  .Code);
              }
            }

            // To-Do: move this out of the sql transaction and chain it?
            data.Stop['StopName'] = 'Stop ' + data.Stop.Sms;

            // Loop through, add extra values
            if (data.Services) {
              for (var i in data.Services) {
                var service = data.Services[i];

                service.MinsAway = moment(service.DisplayDeparture)
                  .diff(moment(data.LastModified), 'minutes');
                service.DisplayDepartureFormatted = moment(service.DisplayDeparture)
                  .format('h:mma');
                if (service.MinsAway <= 2) {
                  service.IsDue = true;
                } else {
                  service.IsDue = false;
                }

                if (favServices.indexOf(service.Service.TrimmedCode) != -1) {
                  service.IsFavourite = true;
                }
              }
            }

            deferred.resolve(data);
          });
        });
      });
    })
    .error(function(data, status) {
      deferred.reject(data);
    });

  return deferred.promise;
}

function getRouteMap(mode, code, direction) {
  var deferred = $q.defer();

  $http.get(ApiEndpointRouteMap + '/' + mode.toLowerCase() + '/' + code.toUpperCase() + '/' + direction.toLowerCase() + '/mapdatajson')
    .success(function(data, status) {

      return deferred.resolve(data);
    })
    .error(function(data, status) {

      deferred.reject(data);
    });

  return deferred.promise;
}

function getServiceLocations(code, direction) {
  var deferred = $q.defer();

  $http.get(ApiEndpoint + '/ServiceLocation/' + code.toUpperCase())
    .success(function(data, status) {

      return deferred.resolve(data);
    })
    .error(function(data, status) {

      deferred.reject(data);
    });

  return deferred.promise;
}

function getAllServices() {
  console.log('getting services');
  var q = $q.defer();

  $ionicPlatform.ready(function() {
    // First see if we need to clear out our data
    Preferences.get('ServicesDataLastUpdated')
      .then(function(val) {
        updateServiceData(q, val);
      }, function(error) {
        updateServiceData(q, false);
      });
  });

  return q.promise;
}

function updateServiceData(q, val) {
  var services = [];
  if (val) {
    var servicesLastUpdated = moment(val);
  }
  var repopulateServices = false;

  if (!servicesLastUpdated || moment()
    .diff(servicesLastUpdated, 'days') >= 1) {
    // Clear our tables so they can be repopulated either now or next time
    repopulateServices = true;
  }

  db.transaction(function(tx) {
    if (repopulateServices) {
      console.log('Cleared Services due to time passed');
      tx.executeSql('DELETE FROM Services');
    }
    tx.executeSql('SELECT s.*, f.Code AS IsFavourite FROM Services as s LEFT JOIN FavouriteServices AS f ON s.Code = f.Code', [], function(tx, rs) {
      if (rs.rows.length > 0) {
        for (var i = 0; i < rs.rows.length; i++) {
          var thisService = angular.copy(rs.rows.item(i));

          if (thisService.IsFavourite) {
            thisService.IsFavourite = true;
          }
          services.push(thisService);
        }

        q.resolve(services);

      } else {
        // Empty DB, so fetch and populate from the API
        $http.get(ApiEndpoint + '/ServiceList/')
          .success(function(data, status) {
            console.log('fetched services from api');

            // Add any additional processing or fields we might want
            // Ensure child services don't appear, it should be the parent stop instead
            apiServices = data;

            for (var i in apiServices) {
              if (apiServices[i]['Mode'] != 'Bus' && apiServices[i]['Mode'] != 'Train' && apiServices[i]['Mode'] != 'School') {
                apiServices[i]['Mode'] = 'Other';
              }
            }

            //console.log(apiServices);
            // Insert into the DB - refetch and return
            db.transaction(function(tx) {
              // Clear the table first
              tx.executeSql('DELETE FROM Services');
              for (var i in apiServices) {
                tx.executeSql('INSERT INTO Services (Name,Code,Mode,LastModified) VALUES (?,?,?,?)', [apiServices[i].Name, apiServices[i].TrimmedCode, apiServices[i].Mode, apiServices[i].LastModified], function(tx, rs) {}, function(tx, error) {
                  console.warn('INSERT error: ' + error.message);
                });
              }

              Preferences.set('ServicesDataLastUpdated', moment()
                .format());

              // If we get them from the API then we haven't joined with our favourites
              // So we need to re-do our select
              tx.executeSql('SELECT s.*, f.Code AS IsFavourite FROM Services as s LEFT JOIN FavouriteServices AS f ON s.Code = f.Code', [], function(tx, rs) {
                console.log('fetched from db after api import: ' + rs.rows.length);
                if (rs.rows.length > 0) {
                  for (var i = 0; i < rs.rows.length; i++) {
                    services.push(angular.copy(rs.rows.item(i)));
                  }
                  q.resolve(services);
                } else {
                  console.warn('Inserted but still no services');
                }
              });
            });

          })
          .error(function(data, status) {
            console.warn('api call failed');
            console.warn(data);
            console.warn(status);
            q.reject(data);
          });
      }
    }, function(tx, error) {
      console.warn('SELECT error: ' + error.message);
      q.reject(error.message);
    });
  });
}

function updateStopData(q, val) {
  var stops = [];
  if (val) {
    var stopsLastUpdated = moment(val);
  }
  var repopulateStops = false;

  if (!stopsLastUpdated || moment()
    .diff(stopsLastUpdated, 'days') >= 1) {
    // Clear our tables so they can be repopulated either now or next time
    repopulateStops = true;
  }

  // Now we've done that, execute the below
  db.transaction(function(tx) {
    if (repopulateStops) {
      console.log('Cleared Stops due to time passed');
      tx.executeSql('DELETE FROM Stops');
    }
    //tx.executeSql('INSERT INTO Stops (Name) VALUES (?)', ['TestName']);
    tx.executeSql('SELECT s.*, f.Sms AS IsFavourite FROM Stops as s LEFT JOIN FavouriteStops AS f ON s.Sms = f.Sms ORDER BY CAST(s.Sms AS INTEGER) ASC', [], function(tx, rs) {
      console.log(rs.rows.length);
      if (rs.rows.length > 0) {
        for (var i = 0; i < rs.rows.length; i++) {
          stops.push(rs.rows.item(i));
        }

        q.resolve(stops);

      } else {
        // Empty DB, so fetch and populate from the API
        $http.get(ApiEndpoint + '/StopList/')
          .success(function(data, status) {
            console.log('fetched stops from api');

            // Add any additional processing or fields we might want
            // Ensure child stops don't appear, it should be the parent stop instead
            apiStops = data.Stops;

            for (var i in apiStops) {
              apiStops[i]['FullName'] = 'Stop ' + apiStops[i].Sms + ' - ' + apiStops[i].Name;

              // If stopSMS is > 4 (i.e. WOBU1 then remove it)
              if (apiStops[i].Sms.length > 4) {
                delete apiStops[i];
              }
            }

            //console.log(apiStops);
            // Insert into the DB - refetch and return
            db.transaction(function(tx) {
              // Clear the table first
              tx.executeSql('DELETE FROM Stops');
              for (var i in apiStops) {
                tx.executeSql('INSERT INTO Stops (Name,Sms,FullName,Farezone,Lat,Long,LastModified) VALUES (?,?,?,?,?,?,?)', [apiStops[i].Name, apiStops[i].Sms, apiStops[i].FullName, apiStops[i].Farezone, apiStops[i].Lat, apiStops[i].Long, apiStops[i].LastModified], function(tx, rs) {}, function(tx, error) {
                  console.warn('INSERT error: ' + error.message);
                });
              }

              Preferences.set('StopDataLastUpdated', moment()
                .format());

              // If we get them from the API then we haven't joined with our favourites
              // So we need to re-do our select
              tx.executeSql('SELECT s.*, f.Sms AS IsFavourite FROM Stops as s LEFT JOIN FavouriteStops AS f ON s.Sms = f.Sms ORDER BY CAST(s.Sms AS INTEGER) ASC', [], function(tx, rs) {
                console.log('fetched from db after api import: ' + rs.rows.length);
                if (rs.rows.length > 0) {
                  for (var i = 0; i < rs.rows.length; i++) {
                    stops.push(rs.rows.item(i));
                  }
                  q.resolve(stops);
                } else {
                  console.warn('Inserted but still no stops');
                }
              });
            });

          })
          .error(function(data, status) {
            console.warn('api call failed');
            console.warn(data);
            console.warn(status);
            q.reject(data);
          });
      }
    }, function(tx, error) {
      console.warn('SELECT error: ' + error.message);
      q.reject(error.message);
    });
  });
}
